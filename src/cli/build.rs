use anyhow::Result;
use log::{debug, error, info, trace, warn};
use std::path::PathBuf;

use super::BuildArgs;
use super::config::{Config, find_default_config};
use crate::site_builder::build_site;

fn load_config(args: &BuildArgs, env_prefix: &str) -> Config {
    let mut config = if let Some(config_path) = &args.config {
        info!("Loading config from: {}", config_path.display());
        match Config::from_file(config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "Error loading config file '{}': {}",
                    config_path.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    } else if let Some(default_config) =
        find_default_config(&args.base_path.clone().unwrap_or(PathBuf::from(".")))
    {
        info!("Auto-detected config file: {}", default_config.display());
        match Config::from_file(&default_config) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!(
                    "Error loading config file '{}': {}",
                    default_config.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    } else {
        warn!("No config file found, using defaults");
        Config::default()
    };
    debug!("Config loaded:\n{:#?}", config);

    // Load config from environment variables and merge (env takes priority over file)
    debug!("Loading environment variables with prefix: {}_", env_prefix);
    let env_config = Config::from_env(env_prefix);
    config = config.merge(env_config);
    trace!("Config after environment variables:\n{:#?}", config);

    // Create config from CLI arguments and merge (CLI takes priority over env)
    debug!("Merging CLI arguments into config");
    let cli_config = match Config::from_cli(args) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Error parsing CLI arguments: {}", e);
            std::process::exit(1);
        }
    };
    config = config.merge(cli_config);
    trace!("Config after CLI arguments:\n{:#?}", config);

    config
}

pub fn handle_build(args: BuildArgs, env_prefix: &str) -> Result<()> {
    let config = load_config(&args, env_prefix);

    // Resolve paths
    let base_path = config.get_base_path();
    let templates_path = config.get_templates_path(&base_path);
    let content_path = config.get_content_path(&base_path);
    let static_path = config.get_static_path(&base_path);
    let output_path = config.get_output_path();

    debug!(
        "Resolved paths:\n  Base: {}\n  Templates: {}\n  Content: {}\n  Static: {}\n  Output: {}",
        base_path.display(),
        templates_path.display(),
        content_path.display(),
        static_path.display(),
        output_path.display(),
    );

    // Clean output directory if requested
    if args.clean {
        if output_path.exists() {
            info!("Cleaning output directory: {}", output_path.display());
            if let Err(e) = std::fs::remove_dir_all(&output_path) {
                error!("Failed to clean output directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    let site_meta: crate::site_builder::models::SiteMeta = config.into();
    trace!("Site metadata prepared:\n{:#?}", site_meta);

    // Build the site
    debug!("Starting site generation...");
    match build_site(
        &content_path,
        &static_path,
        &templates_path,
        &output_path,
        site_meta,
    ) {
        Ok(_) => {
            info!("Site generated successfully!");
            Ok(())
        }
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
