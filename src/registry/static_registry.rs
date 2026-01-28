use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Registry for managing static files and their hashes
#[derive(Default, Debug)]
pub(crate) struct StaticFileRegistry {
    /// Maps absolute paths to relative paths
    pub(crate) files: BTreeMap<PathBuf, PathBuf>,
    /// Maps absolute paths to SHA256 hashes
    pub(crate) hashes: BTreeMap<PathBuf, String>,
}

impl StaticFileRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    
    /// Adds a static file to the registry
    pub(crate) fn push_static(&mut self, path: &Path, prefix: &Path) -> Result<()> {
        let rel_path = path.strip_prefix(prefix)?;
        let bytes = std::fs::read(path)?;
        let hash = sha256::digest(&bytes);
        
        self.hashes.insert(path.to_path_buf(), hash);
        self.files.insert(path.to_path_buf(), rel_path.to_path_buf());
        
        Ok(())
    }

    /// Copies a single static file to the output directory
    pub(crate) fn copy_static(&self, absolute_path: &Path, out_path: &Path) -> Result<()> {
        if let Some(relative) = self.files.get(absolute_path) {
            let file = out_path.join(relative);
            if let Some(dir) = file.as_path().parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::copy(absolute_path, file)?;
            Ok(())
        } else {
            anyhow::bail!("File not found: {:?}", absolute_path)
        }
    }

    /// Copies all static files to the output directory
    pub(crate) fn copy_all(&self, out_path: &Path) -> Result<()> {
        let keys: Vec<PathBuf> = self.files.keys().cloned().collect();
        for absolute_path in keys {
            self.copy_static(&absolute_path, out_path)?
        }
        Ok(())
    }
}
