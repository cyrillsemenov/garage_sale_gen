use anyhow::Result;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::cli::BuildArgs;
use crate::site_builder::models::SiteMeta;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_template: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let config: Self = serde_yaml::from_reader(file)?;
        Ok(config)
    }

    pub fn from_cli(args: &BuildArgs) -> Result<Self> {
        let mut extra = BTreeMap::new();

        let vars = args.parse_vars()?;
        extra.extend(vars);

        let json_vars = args.parse_json_var()?;
        extra.extend(json_vars);

        Ok(Self {
            base_path: args.base_path.clone(),
            templates_path: args.templates_path.clone(),
            static_path: args.static_path.clone(),
            content_path: args.content_path.clone(),
            output_path: args.output_path.clone(),
            title: args.title.clone(),
            locale: args.locale.clone(),
            default_template: Some(args.default_template.clone()),
            extra,
        })
    }

    /// Create configuration from environment variables with a given prefix
    /// Example: PREFIX_TITLE, PREFIX_LOCALE, PREFIX_CUSTOM_VAR
    pub fn from_env(prefix: &str) -> Self {
        use std::env;

        let mut config = Self::default();
        let prefix_upper = prefix.to_uppercase();

        let get_env =
            |key: &str| -> Option<String> { env::var(format!("{}_{}", prefix_upper, key)).ok() };

        if let Some(val) = get_env("BASE_PATH") {
            config.base_path = Some(PathBuf::from(val));
        }
        if let Some(val) = get_env("TEMPLATES_PATH") {
            config.templates_path = Some(PathBuf::from(val));
        }
        if let Some(val) = get_env("STATIC_PATH") {
            config.static_path = Some(PathBuf::from(val));
        }
        if let Some(val) = get_env("CONTENT_PATH") {
            config.content_path = Some(PathBuf::from(val));
        }
        if let Some(val) = get_env("OUTPUT_PATH") {
            config.output_path = Some(PathBuf::from(val));
        }
        if let Some(val) = get_env("TITLE") {
            config.title = Some(val);
        }
        if let Some(val) = get_env("LOCALE") {
            config.locale = Some(val);
        }
        if let Some(val) = get_env("DEFAULT_TEMPLATE") {
            config.default_template = Some(val);
        }

        let prefix_with_underscore = format!("{}_", prefix_upper);
        for (key, value) in env::vars() {
            if let Some(suffix) = key.strip_prefix(&prefix_with_underscore) {
                // Skip known fields (already handled above)
                let suffix_lower = suffix.to_lowercase();
                if !matches!(
                    suffix_lower.as_str(),
                    "base_path"
                        | "templates_path"
                        | "static_path"
                        | "content_path"
                        | "output_path"
                        | "title"
                        | "locale"
                        | "default_template"
                ) {
                    config
                        .extra
                        .insert(suffix_lower, serde_yaml::Value::String(value));
                }
            }
        }

        config
    }

    pub fn merge(mut self, other: Config) -> Self {
        if other.base_path.is_some() {
            self.base_path = other.base_path;
        }
        if other.templates_path.is_some() {
            self.templates_path = other.templates_path;
        }
        if other.static_path.is_some() {
            self.static_path = other.static_path;
        }
        if other.content_path.is_some() {
            self.content_path = other.content_path;
        }
        if other.output_path.is_some() {
            self.output_path = other.output_path;
        }
        if other.title.is_some() {
            self.title = other.title;
        }
        if other.locale.is_some() {
            self.locale = other.locale;
        }
        if other.default_template.is_some() {
            self.default_template = other.default_template;
        }

        self.extra.extend(other.extra);

        self
    }

    pub fn get_base_path(&self) -> PathBuf {
        self.base_path.clone().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn get_templates_path(&self, base: &Path) -> PathBuf {
        self.templates_path
            .clone()
            .unwrap_or_else(|| base.join("templates"))
    }

    pub fn get_static_path(&self, base: &Path) -> PathBuf {
        self.static_path
            .clone()
            .unwrap_or_else(|| base.join("static"))
    }

    pub fn get_content_path(&self, base: &Path) -> PathBuf {
        self.content_path
            .clone()
            .unwrap_or_else(|| base.join("content"))
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./public"))
    }
}

impl Into<SiteMeta> for Config {
    fn into(self) -> SiteMeta {
        let mut extra = self.extra;

        if let Some(title) = self.title {
            extra.insert("title".to_string(), serde_yaml::Value::String(title));
        }
        if let Some(locale) = self.locale {
            extra.insert("locale".to_string(), serde_yaml::Value::String(locale));
        }

        SiteMeta {
            extra,
            default_template: self.default_template,
            navigation: Vec::new(),
        }
    }
}

pub fn find_default_config(base_path: &Path) -> Option<PathBuf> {
    for ext in ["yaml", "yml"] {
        let path = base_path.join(format!("config.{}", ext));
        if path.exists() {
            return Some(path);
        }
    }

    None
}
