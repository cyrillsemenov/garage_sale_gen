use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use yaml_front_matter::Document;

use crate::graph;
use crate::registry::{HasChildrenRoots, IsIndex};

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_template() -> String {
    "base".to_string()
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub(crate) struct RenderingCtx {
    pub(crate) site: SiteMeta,
    pub(crate) page: PageMeta,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub(crate) struct SiteMeta {
    #[serde(default)]
    pub(crate) default_template: Option<String>,
    #[serde(default, skip_deserializing)]
    pub(crate) navigation: Vec<NavNode>,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub(crate) struct NavNode {
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) children: Vec<NavNode>,
    pub(crate) order: Option<i32>,
}

impl SiteMeta {
    pub(crate) fn from_str(s: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(s)?)
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub(crate) struct PageMeta {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) stem: String,
    #[serde(default)]
    pub(crate) template: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) publish: bool,
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) children: Vec<PageMeta>,
    #[serde(default)]
    pub(crate) path_segments: Vec<String>,
    #[serde(default)]
    pub(crate) html_content: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, serde_yaml::Value>,
}

impl PageMeta {
    pub(crate) fn insert_path_segments(&mut self, path_segments: &[String]) {
        self.path_segments.extend_from_slice(path_segments);
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub(crate) struct FileMeta {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) template: Option<String>,
    #[serde(default)]
    pub(crate) publish: Option<bool>,
    #[serde(default)]
    pub(crate) children: Vec<String>,
    #[serde(default)]
    pub(crate) children_attrs: BTreeMap<String, serde_yaml::Value>,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, serde_yaml::Value>,
}

impl FileMeta {
    pub(crate) fn append(&mut self, other: BTreeMap<String, serde_yaml::Value>) -> Result<()> {
        let mapping: serde_yaml::Mapping = other
            .iter()
            .filter(|(k, _)| !matches!(k.to_lowercase().as_str(), "title" | "children_attrs"))
            .map(|(k, v)| (serde_yaml::Value::String(k.to_lowercase()), v.clone()))
            .collect();
        let mut patch: FileMeta = serde_yaml::from_value(serde_yaml::Value::Mapping(mapping))?;

        for k in self.extra.keys() {
            if patch.extra.contains_key(k) {
                patch.extra.remove(k);
            }
        }
        self.extra.extend(patch.extra);

        if self.template.is_none() {
            self.template = patch.template;
        }

        if self.publish.is_none() && patch.publish.is_some() {
            self.publish = patch.publish
        }
        Ok(())
    }
}

impl Into<PageMeta> for FileMeta {
    fn into(self) -> PageMeta {
        PageMeta {
            title: self.title,
            stem: String::new(),
            template: self.template,
            publish: self.publish.unwrap_or_else(default_true),
            children: Vec::new(),
            path_segments: Vec::new(),
            html_content: String::new(),
            url: String::new(),
            id: String::new(),
            extra: self.extra,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PageSource {
    pub(crate) abs_path: PathBuf,
    pub(crate) rel_path: PathBuf,
    pub(crate) parent_dir: PathBuf,
    pub(crate) stem: String,
    pub(crate) file_meta: FileMeta,
    pub(crate) page_meta: Option<PageMeta>,
    pub(crate) content: String,
    pub(crate) hash: String,
}

impl PageSource {
    /// Creates a PageSource from an absolute path relative to a base directory
    pub(crate) fn new(abs_path: &Path, base_dir: &Path) -> Result<Self> {
        use crate::error::{ErrorKind, ParserError};

        // Compute relative path from absolute path and base directory
        let rel_path = abs_path.strip_prefix(base_dir).map_err(|_| {
            ParserError::path_not_under_base(abs_path.to_path_buf(), base_dir.to_path_buf())
        })?;

        let parent_dir = rel_path
            .parent()
            .ok_or_else(|| ParserError {
                kind: ErrorKind::NoParentDir {
                    path: rel_path.to_path_buf(),
                },
                file: Some(abs_path.to_path_buf()),
                span: None,
                source_content: None,
            })?
            .to_path_buf();

        let stem = rel_path
            .file_stem()
            .ok_or_else(|| ParserError {
                kind: ErrorKind::NoFileStem {
                    path: rel_path.to_path_buf(),
                },
                file: Some(abs_path.to_path_buf()),
                span: None,
                source_content: None,
            })?
            .to_string_lossy()
            .to_string();

        // Read file with detailed error reporting
        let mut raw_file = String::new();
        File::open(abs_path)
            .map_err(|e| ParserError::file_not_found(abs_path.to_path_buf(), e))?
            .read_to_string(&mut raw_file)
            .map_err(|_| ParserError::invalid_utf8(abs_path.to_path_buf()))?;

        let hash = sha256::digest(&raw_file);

        // Parse front matter with error context
        let fm_result: std::result::Result<Document<FileMeta>, _> =
            yaml_front_matter::YamlFrontMatter::parse(&raw_file);

        let (file_meta, content) = match fm_result {
            Ok(doc) => (doc.metadata, doc.content),
            Err(e) => {
                // Try to find the front matter section for better error reporting
                let fm_end = raw_file
                    .find("---")
                    .and_then(|start| raw_file[start + 3..].find("---").map(|end| start + end + 6))
                    .unwrap_or(raw_file.len().min(100));

                return Err(ParserError::invalid_front_matter(
                    abs_path.to_path_buf(),
                    e.to_string(),
                    raw_file.clone(),
                    (0, fm_end),
                )
                .into());
            }
        };

        Ok(Self {
            abs_path: abs_path.to_path_buf(),
            rel_path: rel_path.to_path_buf(),
            parent_dir,
            stem,
            file_meta,
            page_meta: None,
            content,
            hash,
        })
    }
}

impl IsIndex for PageSource {
    fn is_index(&self) -> bool {
        self.stem == "index"
    }
}

// impl HasParentDir for PageSource {
//     fn parent_dir(&self) -> &Path {
//         &self.parent_dir
//     }
// }

impl HasChildrenRoots for PageSource {
    fn children_roots(&self) -> &[String] {
        &self.file_meta.children
    }
}

impl graph::GraphNode for PageSource {
    fn child_roots(&self) -> &[String] {
        &self.file_meta.children
    }

    fn is_index(&self) -> bool {
        self.stem == "index"
    }

    fn parent_dir(&self) -> &Path {
        &self.parent_dir
    }
}
