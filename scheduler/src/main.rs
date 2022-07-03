#![forbid(unsafe_code)]
use std::path::PathBuf;

use clap::{ArgEnum, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, error};

mod commands;

#[derive(Debug, Parser)]
/// RCPSP scheduler
struct App {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a graphviz notation for specific psp lib problem
    Graph(Graph),
    /// Create a schedule for a given psp lib problem
    Schedule(Schedule),
    /// Run benchmarks for scheduler
    Benchmark(Benchmark),
}

#[derive(Debug, Parser)]
pub struct Graph {
    #[clap(required = true, parse(from_os_str))]
    psp_problem_file: PathBuf,
    #[clap(required = true, parse(from_os_str))]
    output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct Benchmark {
    #[clap(parse(from_os_str))]
    psp_problem_file_folder: PathBuf,
    #[clap(parse(from_os_str))]
    output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct Schedule {
    /// Instances data
    #[clap(required = true, parse(from_os_str), min_values = 1)]
    input_files: Vec<PathBuf>,

    /// Run scheduler multi-threaded
    #[clap(long, short = 'p')]
    parallel: bool,

    /// Type of the tabu list to be used
    #[clap(arg_enum, long, visible_alias = "mode", default_value_t = Mode::Simple)]
    tabu_list_mode: Mode,

    /// If you want to write makespan criterion graph (independent variable is
    /// number of iterations)
    #[clap(long, visible_alias = "wmg")]
    write_makespan_graph: bool,

    /// Add this option if you want to write a file with the best schedule.
    /// This file is binary.
    #[clap(long, visible_alias = "wrf")]
    write_result_file: bool,

    /// Number of iterations after which the search process will be stopped.
    #[clap(long, visible_alias = "noi", default_value_t = 1000)]
    number_of_iterations: u32,

    /// Maximal number of iterations without improving solution after which
    /// diversification is called.
    #[clap(long, visible_alias = "misb", default_value_t = 300)]
    max_iter_since_best: u32,
    /// Number of iterations without finding an improved solution
    /// after which tabu search will begin at the initial solution again
    #[clap(long, visible_alias = "isbr")]
    iter_since_best_reset: Option<u32>,
    /// Size of the simple tabu list. Ignored for the advanced tabu list.
    #[clap(long, visible_alias = "tls", default_value_t = 800)]
    tabu_list_size: u32,

    /// Maximal distance between swapped activities.
    #[clap(long, visible_alias = "swr", default_value_t = 60)]
    swap_range: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ArgEnum)]
pub enum Mode {
    /// The simple version of the tabu list is used.
    Simple,
    /// More sophisticated version of the tabu list is used.
    Advanced,
}

fn main() {
    let args: App = App::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    debug!("{args:#?}");

    if let Err(err) = match args.command {
        Commands::Graph(Graph {
            psp_problem_file,
            output,
        }) => commands::graph(psp_problem_file, output),
        Commands::Schedule(schedule) => commands::schedule(schedule),
        Commands::Benchmark(benchmark) => commands::benchmark(benchmark),
    } {
        error!("An error occurred: {}", err);
    }
}
