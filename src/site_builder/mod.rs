use crate::error::{ErrorCollector, ParserError};
use crate::graph::{Pages, UnderRoot};
use crate::registry::{IsIndex, PageRegistry, StaticFileRegistry};
use crate::utils::FileCollector;
use anyhow::Result;
use log::{debug, trace};
use models::PageSource;
pub(crate) use page_id::PageId;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub mod models;
pub mod page_id;

#[derive(Default, Debug)]
pub struct SiteBuilder {
    content_dir: PathBuf,
    page_registry: PageRegistry,
    static_registry: StaticFileRegistry,
}

impl SiteBuilder {
    pub fn new(content_dir: &Path) -> Self {
        let mut this = Self::default();
        this.content_dir = content_dir.to_path_buf();
        this
    }

    /// Creates a SiteBuilder by collecting files from content and static directories
    /// Returns the builder and an ErrorCollector with any errors encountered
    pub fn from_directories(
        content_dir: &Path,
        static_dir: &Path,
    ) -> Result<(Self, ErrorCollector)> {
        debug!(
            "Collecting files from content: {}, static: {}",
            content_dir.display(),
            static_dir.display()
        );
        let mut builder = Self::new(content_dir);
        let mut errors = ErrorCollector::new();

        if let Err(e) = Self::collect_static_files(&mut builder, static_dir) {
            errors.add_error(e.into());
        }

        Self::collect_content_files_with_errors(&mut builder, content_dir, &mut errors);

        debug!(
            "Collected {} pages, {} static files",
            builder.page_registry.pages.len(),
            builder.static_registry.files.len()
        );

        Ok((builder, errors))
    }

    fn collect_static_files(builder: &mut Self, static_dir: &Path) -> Result<()> {
        let prefix = static_dir.canonicalize()?;
        let mut count = 0;
        for path in FileCollector::new(static_dir, true)?.filter(|p| !Self::is_hidden(p)) {
            trace!("Collecting static file: {}", path.display());
            builder.push_static(&path, &prefix)?;
            count += 1;
        }
        debug!("Collected {} static files", count);
        Ok(())
    }

    fn collect_content_files_with_errors(
        builder: &mut Self,
        content_dir: &Path,
        errors: &mut ErrorCollector,
    ) {
        let prefix = match content_dir.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                errors.add_error(ParserError::file_not_found(content_dir.to_path_buf(), e));
                return;
            }
        };

        let mut file_collector = match FileCollector::new(content_dir, true) {
            Ok(fc) => fc,
            Err(e) => {
                errors.add_error(e.into());
                return;
            }
        };

        for path in file_collector.filter(|p| !Self::is_hidden(p)) {
            if Self::is_markdown(&path) {
                trace!("Processing markdown file: {}", path.display());
                match PageSource::new(&path, &prefix) {
                    Ok(page) => {
                        if let Err(e) = builder.push_page(&path, &prefix, page) {
                            errors.add_error(e.into());
                        }
                    }
                    Err(e) => {
                        errors.add_error(e.into());
                    }
                }
            } else {
                trace!("Processing static file: {}", path.display());
                if let Err(e) = builder.push_static(&path, &prefix) {
                    errors.add_error(e.into());
                }
            }
        }
    }

    fn is_hidden(p: &Path) -> bool {
        p.file_name()
            .is_some_and(|s| s.to_string_lossy().starts_with("."))
    }

    fn is_markdown(p: &Path) -> bool {
        p.extension().is_some_and(|s| s == "md")
    }
}

impl UnderRoot<PageId> for SiteBuilder {
    fn under_root(&self, root: &Path) -> Vec<(PageId, Vec<String>)> {
        let mut out = Vec::new();
        for (id, src) in &self.page_registry.pages {
            if src.rel_path.starts_with(root) {
                // Exclude ONLY the index at the root itself
                let is_root_index = src.is_index() && src.parent_dir == root;
                if is_root_index {
                    continue;
                }

                // path segments = components of the parent directory relative to content root
                // effectively mirroring the physical structure
                let mut segs = Vec::new();
                for comp in src.rel_path.parent().unwrap_or(Path::new("")).iter() {
                    let s = comp.to_string_lossy();
                    if !s.is_empty() && s != "." {
                        segs.push(s.to_string());
                    }
                }
                out.push((id.clone(), segs));
            }
        }
        out
    }
}

impl Pages<PageId, PageSource> for SiteBuilder {
    fn get_pages(&self) -> &BTreeMap<PageId, PageSource> {
        &self.page_registry.pages
    }
}

impl SiteBuilder {
    pub(crate) fn push_page(&mut self, path: &Path, prefix: &Path, page: PageSource) -> Result<()> {
        self.page_registry.push_page(path, prefix, page)
    }

    pub(crate) fn push_static(&mut self, path: &Path, prefix: &Path) -> Result<()> {
        self.static_registry.push_static(path, prefix)
    }

    pub(crate) fn copy_static(&self, absolute_path: &Path, out_path: &Path) -> Result<()> {
        self.static_registry.copy_static(absolute_path, out_path)
    }

    pub(crate) fn copy_all(&self, out_path: &Path) -> Result<()> {
        debug!("Copying all static files to: {}", out_path.display());
        self.static_registry.copy_all(out_path)
    }

    pub(crate) fn pages(&self) -> &BTreeMap<PageId, PageSource> {
        &self.page_registry.pages
    }

    pub(crate) fn pages_mut(&mut self) -> &mut BTreeMap<PageId, PageSource> {
        &mut self.page_registry.pages
    }
}

pub fn build_site(
    content_path: &Path,
    static_path: &Path,
    templates_path: &Path,
    output_path: &Path,
    site_meta: models::SiteMeta,
) -> Result<()> {
    use crate::processor::Processor;
    use crate::renderer::Renderer;

    let (mut builder, mut errors) = SiteBuilder::from_directories(content_path, static_path)?;

    let mut processor = Processor::new(&builder, &mut errors)?;

    errors.print_all();
    errors.into_result().map_err(|e| anyhow::anyhow!("{}", e))?;

    processor.process(&mut builder)?;

    let mut site_meta = site_meta;
    site_meta.navigation = processor.build_navigation(&builder)?;

    let breadcrumbs = processor.compute_breadcrumbs(&builder)?;

    let mut urls = std::collections::BTreeMap::new();
    for (id, page) in builder.pages() {
        if let Some(meta) = &page.page_meta {
            urls.insert(id.clone(), meta.url.clone());
        }
    }

    let mut renderer = Renderer::new(templates_path, output_path, site_meta, breadcrumbs, urls)?;

    renderer.render_all(&mut builder)?;

    Ok(())
}
