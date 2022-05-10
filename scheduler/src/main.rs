#![forbid(unsafe_code)]
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use log::{debug, error};

mod commands;

#[derive(Debug, Parser)]
/// RCPSP scheduler
struct App {
    #[clap(flatten)]
    verbose: Verbosity,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a graphviz notation for specific psp lib problem
    Graph {
        #[clap(required = true, parse(from_os_str))]
        psp_problem_file: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Create a schedule for a given psp lib problem
    Schedule {
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
    },
}

fn main() {
    let args: App = App::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    debug!("{args:?}");

    if let Err(err) = match args.command {
        Commands::Graph {
            psp_problem_file,
            output,
        } => commands::graph(psp_problem_file, output),
        Commands::Schedule { path } => commands::schedule(path),
    } {
        error!("An error occurred: {}", err);
    }
}
