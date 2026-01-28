use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::site_builder::PageId;
use crate::site_builder::models::PageSource;

/// Registry for managing pages and their directory indices
#[derive(Default, Debug)]
pub(crate) struct PageRegistry {
    /// All pages indexed by their ID
    pub(crate) pages: BTreeMap<PageId, PageSource>,
    /// Index of page IDs by their parent directory
    pub(crate) by_dir: BTreeMap<PathBuf, BTreeSet<PageId>>,
    /// Directories that contain index pages (sections)
    pub(crate) section_dirs: BTreeSet<PathBuf>,
}

impl PageRegistry {
    /// Adds a page to the registry
    pub(crate) fn push_page(&mut self, path: &Path, prefix: &Path, page: PageSource) -> Result<()> {
        let rel_path = path.strip_prefix(prefix).unwrap();
        let page_id: PageId = rel_path.to_path_buf().into();
        let parent = &page.parent_dir;

        // Add to directory index
        self.by_dir
            .entry(parent.to_path_buf())
            .or_default()
            .insert(page_id.clone());

        // Mark as section if it's an index page
        if page.is_index() {
            self.section_dirs.insert(parent.to_path_buf());
        }

        // Register child collection roots as sections
        for c in page.children_roots() {
            let section = prefix.join(parent).join(c);
            if section.is_dir() {
                self.section_dirs.insert(parent.join(c));
            }
        }

        // Store the page
        self.pages.insert(page_id, page);

        Ok(())
    }

    // /// Gets a reference to all pages
    // pub(crate) fn pages(&self) -> &BTreeMap<PageId, PageSource> {
    //     &self.pages
    // }

    // /// Gets a mutable reference to all pages
    // pub(crate) fn pages_mut(&mut self) -> &mut BTreeMap<PageId, PageSource> {
    //     &mut self.pages
    // }
}

pub(crate) trait IsIndex {
    fn is_index(&self) -> bool;
}

// pub(crate) trait HasParentDir {
//     fn parent_dir(&self) -> &Path;
// }

pub(crate) trait HasChildrenRoots {
    fn children_roots(&self) -> &[String];
}
