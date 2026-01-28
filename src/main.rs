use clap::Parser;
use log::{error, trace};

mod cli;
mod error;
mod graph;
mod processor;
mod registry;
mod renderer;
mod site_builder;
mod utils;

fn main() {
    let cli = cli::Cli::parse();
    utils::init_logger(cli.silent, cli.verbose, &cli.env_prefix);

    trace!("CLI arguments:\n{:#?}", cli);

    if let Err(e) = cli::handle_command(cli) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
