use log::LevelFilter;
use std::env;

pub(crate) fn init_logger(silent: bool, verbose: u8, env_prefix: &str) {
    let log_level = if silent {
        LevelFilter::Error
    } else if verbose > 0 {
        match verbose {
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    } else {
        let env_log_var = format!("{}_LOG", env_prefix.to_uppercase());
        if let Ok(level_str) = env::var(&env_log_var) {
            match level_str.to_lowercase().as_str() {
                "trace" | "verbose" => LevelFilter::Trace,
                "debug" => LevelFilter::Debug,
                "info" => LevelFilter::Info,
                "warn" | "warning" => LevelFilter::Warn,
                "error" => LevelFilter::Error,
                "0" | "off" | "none" | "silent" => LevelFilter::Off,
                _ => {
                    eprintln!(
                        "Warning: Invalid log level '{}' in {}, using default (warn)",
                        level_str, env_log_var
                    );
                    LevelFilter::Warn
                }
            }
        } else {
            LevelFilter::Warn
        }
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();
}
