use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) trait PageRegistry {
    type PageId: Ord + Clone + From<PathBuf>;
    type Page: IsIndex + HasParentDir + HasChildrenRoots;

    fn page_map_insert(&mut self, key: Self::PageId, value: Self::Page) -> Option<Self::Page>;
    fn page_id_by_dir_insert(&mut self, key: &Path, value: &Self::PageId) -> &'_ mut BTreeSet<Self::PageId>;
    fn section_dirs_insert(&mut self, value: &Path) -> bool;

    fn push_page(&mut self, path: &Path, prefix: &Path, page: Self::Page) -> Result<()> {
        let rel_path = path.strip_prefix(prefix).unwrap();
        let page_id: Self::PageId = rel_path.to_path_buf().into();
        let parent = page.parent_dir();

        self.page_id_by_dir_insert(parent, &page_id);
        if page.is_index() {
            self.section_dirs_insert(parent);
        }

        for c in page.children_roots() {
            let section = prefix.join(&parent).join(c);
            if section.is_dir() {
                self.section_dirs_insert(&parent.join(c));
            }
        }
        self.page_map_insert(page_id.clone(), page);

        Ok(())
    }
}

pub(crate) trait IsIndex {
    fn is_index(&self) -> bool;
}

pub(crate) trait HasParentDir {
    fn parent_dir(&self) -> &Path;
}

pub(crate) trait HasChildrenRoots {
    fn children_roots(&self) -> &[String];
}
