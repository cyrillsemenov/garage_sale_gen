use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use tera::{Context, Tera};

// use crate::error::{ErrorCollector, ParserError};
use crate::site_builder::SiteBuilder;
use crate::site_builder::models::{PageMeta, PageSource, RenderingCtx, SiteMeta};

mod tera_filters;
mod tera_functions;

pub struct Renderer {
    tera: Tera,
    site_meta: SiteMeta,
    output_dir: PathBuf,
}

const TEMPLATE_EXTENSIONS: [&str; 4] = ["html", "html.j2", "jinja2", "j2"];

impl Renderer {
    pub fn new(
        template_dir: &Path,
        output_dir: &Path,
        site_meta: SiteMeta,
        breadcrumbs: BTreeMap<
            crate::site_builder::PageId,
            Vec<crate::site_builder::models::NavNode>,
        >,
        urls: BTreeMap<crate::site_builder::PageId, String>,
    ) -> Result<Self> {
        let pattern = template_dir.join("**/*");
        let pattern_str = pattern.to_str().ok_or_else(|| {
            anyhow::anyhow!("Template path contains invalid UTF-8: {:?}", pattern)
        })?;

        let mut tera = Tera::new(pattern_str)?;
        tera.autoescape_on(vec![]);

        use tera_filters::*;
        tera.register_filter("word_to_color", word_to_color_filter);
        tera.register_filter("word_text_color", word_text_color_filter);
        tera.register_filter("word_hsl", word_hsl_filter);
        tera.register_filter("pop", pop_filter);
        tera_text_filters::register_all(&mut tera);

        use tera_functions::*;
        tera.register_function("url_for", make_url_for(urls));
        tera.register_function("breadcrumbs", make_breadcrumbs(breadcrumbs));

        Ok(Self {
            tera,
            output_dir: output_dir.to_path_buf(),
            site_meta,
        })
    }

    pub fn prepare_output(&self) -> Result<()> {
        if self.output_dir.exists() {
            std::fs::remove_dir_all(&self.output_dir)?;
        }
        std::fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }

    pub fn render_all(&mut self, builder: &mut SiteBuilder) -> Result<()> {
        for (page_id, page) in builder.pages() {
            if let Some(meta) = &page.page_meta {
                // Skip unpublished pages (metadata-only children)
                if !meta.publish {
                    continue;
                }
                self.render_page(page)?;
            }
        }
        builder.copy_all(&self.output_dir)?;
        Ok(())
    }

    /// Resolves template name to actual template file
    /// Supports both explicit extensions and extension-less names
    fn resolve_template_name(&self, template: &str) -> Result<String> {
        // If template already has extension, use it directly
        if template.contains('.') {
            if self.tera.get_template(template).is_ok() {
                return Ok(template.to_string());
            }
            return Err(anyhow::anyhow!(
                "Template '{}' not found (explicit extension provided)",
                template
            ));
        }

        let mut found = Vec::new();

        for ext in TEMPLATE_EXTENSIONS {
            let name = format!("{}.{}", template, ext);
            if self.tera.get_template(&name).is_ok() {
                found.push(name);
            }
        }

        match found.len() {
            0 => Err(anyhow::anyhow!(
                "Template '{}' not found. Tried: {}",
                template,
                TEMPLATE_EXTENSIONS
                    .iter()
                    .map(|e| format!("{}.{}", template, e))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            1 => Ok(found[0].clone()),
            _ => {
                // Multiple matches - use first but warn
                use owo_colors::OwoColorize;
                eprintln!(
                    "{} Multiple templates found for '{}': {}. Using '{}'",
                    "Warning:".yellow().bold(),
                    template,
                    found.join(", "),
                    found[0]
                );
                Ok(found[0].clone())
            }
        }
    }

    pub fn render_page(&mut self, page: &PageSource) -> Result<()> {
        let Some(meta) = &page.page_meta else {
            return Ok(());
        };

        // Determine template name: Page > Site Default > "base" fallback
        let template_key = meta
            .template
            .as_deref()
            .or(self.site_meta.default_template.as_deref())
            .unwrap_or("base");

        // Resolve template name with smart extension handling
        let template_name = self.resolve_template_name(template_key)?;
        let context = self.create_context(meta)?;

        match self.tera.render(&template_name, &context) {
            Ok(html) => self.write_output(page, html)?,
            Err(e) => eprintln!("Render error: {:?}", e),
        }

        Ok(())
    }

    fn create_context(&self, meta: &PageMeta) -> Result<Context> {
        let ctx = RenderingCtx {
            site: self.site_meta.clone(),
            page: meta.clone(),
        };
        Ok(Context::from_serialize(ctx)?)
    }

    fn write_output(&self, page: &PageSource, html: String) -> Result<()> {
        let output_file = self.output_dir.join(page.rel_path.with_extension("html"));

        if let Some(dir) = output_file.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let mut file = File::create(output_file)?;
        file.write_all(html.as_bytes())?;

        Ok(())
    }
}
