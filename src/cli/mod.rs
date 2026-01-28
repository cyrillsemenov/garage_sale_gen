use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::collections::BTreeMap;
use std::path::PathBuf;

mod build;
pub(self) mod config;
mod scaffold;

#[derive(Parser, Debug)]
#[command(name = "build")]
#[command(about = "Generate a static site from markdown content", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level: -v for info, -vv for debug, -vvv for trace
    #[arg(short, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Silent
    #[arg(short, long, global = true)]
    pub silent: bool,

    /// Environment variable prefix for loading config from env vars
    #[arg(long, value_name = "PREFIX", default_value = "SITE", global = true)]
    pub env_prefix: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build the static site
    Build(BuildArgs),
    /// Scaffold a new site from example
    Scaffold(ScaffoldArgs),
}

#[derive(Args, Debug)]
pub struct ScaffoldArgs {
    /// Directory to create the site in (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Example to use (default: "basic")
    #[arg(long, default_value = "basic")]
    pub example: String,

    /// List available examples
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Path to configuration file (default: auto-detect ./config.yaml or ./config.yml)
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Base directory path
    #[arg(long, value_name = "PATH")]
    pub base_path: Option<PathBuf>,

    /// Templates directory path
    #[arg(long, value_name = "PATH")]
    pub templates_path: Option<PathBuf>,

    /// Static files directory path
    #[arg(long, value_name = "PATH")]
    pub static_path: Option<PathBuf>,

    /// Content directory path
    #[arg(long, value_name = "PATH")]
    pub content_path: Option<PathBuf>,

    /// Output directory path
    #[arg(long, value_name = "PATH")]
    pub output_path: Option<PathBuf>,

    /// Site title
    #[arg(long)]
    pub title: Option<String>,

    /// Site locale
    #[arg(long)]
    pub locale: Option<String>,

    /// Default template name (default: "base.html")
    /// Example: --default-template "main.html"
    #[arg(long, default_value = "base.html")]
    pub default_template: String,

    /// Arbitrary key-value pairs (can be used multiple times)
    /// Example: --var author="John Doe" --var date="2024-01-01"
    #[arg(long, value_name = "KEY=VALUE")]
    pub var: Vec<String>,

    /// Arbitrary JSON object to merge into site config
    /// Example: --json-var '{"social": {"twitter": "@example"}}'
    #[arg(long, value_name = "JSON")]
    pub json_var: Option<String>,

    /// Clean output directory before building
    #[arg(short, long)]
    pub clean: bool,
}

impl BuildArgs {
    pub fn parse_vars(&self) -> Result<BTreeMap<String, serde_yaml::Value>> {
        let mut map = BTreeMap::new();

        for var in &self.var {
            let parts: Vec<&str> = var.splitn(2, '=').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid --var format: '{}'. Expected KEY=VALUE", var);
            }

            let key = parts[0].to_string();
            let value = parts[1].to_string();

            map.insert(key, serde_yaml::Value::String(value));
        }

        Ok(map)
    }

    pub fn parse_json_var(&self) -> Result<BTreeMap<String, serde_yaml::Value>> {
        if let Some(json_str) = &self.json_var {
            // let json_value: serde_json::Value = serde_json::from_str(json_str)?;

            // let yaml_str = serde_json::to_string(&json_value)?;
            // let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;

            // This should work bc json is always (!?) valid yaml.
            // But just in case lets keep clunky implementation above.
            // YES, it can parse yaml, but lets not document it. Let the bunny deliver this surprise.
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&json_str)?;

            if let serde_yaml::Value::Mapping(mapping) = yaml_value {
                let mut map = BTreeMap::new();
                for (k, v) in mapping {
                    if let serde_yaml::Value::String(key) = k {
                        map.insert(key, v);
                    }
                }
                return Ok(map);
            } else {
                anyhow::bail!("--json-var must be a JSON object, not an array or primitive");
            }
        }

        Ok(BTreeMap::new())
    }
}

pub fn handle_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Build(args) => build::handle_build(args, &cli.env_prefix),
        Commands::Scaffold(args) => scaffold::handle_scaffold(args),
    }
}
