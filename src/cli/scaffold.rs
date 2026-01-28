use anyhow::Result;
use include_dir::{Dir, include_dir};
use log::{debug, error, info};
use std::path::{Path, PathBuf};

use crate::cli::ScaffoldArgs;

static EXAMPLES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/examples");

pub fn handle_scaffold(args: ScaffoldArgs) -> Result<()> {
    if args.list {
        println!("Available examples:");
        for entry in EXAMPLES_DIR.dirs() {
            if let Some(name) = entry.path().file_name() {
                println!("  - {}", name.to_string_lossy());
            }
        }
        return Ok(());
    }

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(&args.example));
    let target_path = if let Some(path) = args.path {
        current_dir.join(path)
    } else {
        current_dir
    };
    let example_name = &args.example;

    debug!(
        "Scaffolding '{}' to {}",
        example_name,
        target_path.display()
    );

    if target_path.exists() {
        match std::fs::read_dir(&target_path) {
            Ok(read_dir) => {
                if read_dir.count() > 0 {
                    error!("{}", target_path.display());
                    std::process::exit(1);
                }
            }
            Err(e) => {
                error!("Error checking target directory: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        if let Err(e) = std::fs::create_dir_all(&target_path) {
            error!("Failed to create target directory: {}", e);
            std::process::exit(1);
        }
    }

    let example_dir = match EXAMPLES_DIR.get_dir(example_name) {
        Some(dir) => dir,
        None => {
            error!("Example '{}' not found. Available examples: ", example_name);
            for entry in EXAMPLES_DIR.dirs() {
                if let Some(name) = entry.path().file_name() {
                    eprintln!("  - {}", name.to_string_lossy());
                }
            }
            std::process::exit(1);
        }
    };

    debug!("Found example '{}'. Extracting...", example_name);

    fn extract_dir(dir: &Dir, base_path: &Path, strip_prefix: &str) -> std::io::Result<()> {
        for file in dir.files() {
            let path = file.path();
            let relative_path = path.strip_prefix(strip_prefix).unwrap_or(path);

            let target = base_path.join(relative_path);

            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }

            debug!("Writing file: {}", target.display());
            std::fs::write(target, file.contents())?;
        }

        for subdir in dir.dirs() {
            extract_dir(subdir, base_path, strip_prefix)?;
        }
        Ok(())
    }

    if let Err(e) = extract_dir(example_dir, &target_path, example_name) {
        error!("Failed to extract example: {}", e);
        std::process::exit(1);
    }

    info!(
        "Successfully created new site from '{}' example at {}",
        example_name,
        target_path.display()
    );

    Ok(())
}
