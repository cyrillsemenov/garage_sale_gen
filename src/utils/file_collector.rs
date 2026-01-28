use anyhow::Result;

use std::path::Path;

use std::collections::VecDeque;

use std::path::PathBuf;

/// I need this file collector to get files in specific order:
/// first files from root dir, then from subdirs
/// (files in subdirs inside subdirs go first, before next subdirs are being processed).
/// Have i told smth about subdirs? Oh yeah! When recursive is false, subdirs are not visited.
/// (I am not sure is it is useful though)
pub(crate) struct FileCollector {
    pub(crate) base_path: PathBuf,
    pub(crate) dirs_to_visit: VecDeque<PathBuf>,
    pub(crate) collected_files: VecDeque<PathBuf>,
    pub(crate) recursive: bool,
}

impl FileCollector {
    pub(crate) fn new(base_path: &Path, recursive: bool) -> Result<Self> {
        let mut this = Self {
            base_path: base_path.canonicalize()?,
            dirs_to_visit: VecDeque::new(),
            collected_files: VecDeque::new(),
            recursive,
        };
        this.dirs_to_visit.push_front(this.base_path.clone());
        Ok(this)
    }
}

impl Iterator for &mut FileCollector {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.collected_files.is_empty() {
            return self.collected_files.pop_front();
        }
        while let Some(dir) = self.dirs_to_visit.pop_front() {
            for entry in dir.read_dir().expect("read_dir call failed") {
                if let Ok(entry) = entry {
                    if let Ok(p) = entry.path().canonicalize() {
                        if p.is_file() {
                            self.collected_files.push_front(p);
                        } else if p.is_dir() && self.recursive {
                            self.dirs_to_visit.push_front(p);
                        }
                    }
                }
            }
        }
        if !self.collected_files.is_empty() {
            return self.collected_files.pop_front();
        }
        // Allow to recollect files again
        self.dirs_to_visit.push_front(self.base_path.clone());
        return None;
    }
}
