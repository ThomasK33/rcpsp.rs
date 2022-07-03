use anyhow::Result;
use psp_lib_parser::parse_psp_lib;
use rcpsp::scheduler;

use crate::Benchmark;

pub fn benchmark(benchmark: Benchmark) -> Result<()> {
    if !benchmark.psp_problem_file_folder.is_dir() {
        anyhow::bail!("psp_problem_file_folder is not a directory")
    }

    let folder = benchmark.psp_problem_file_folder.read_dir()?;

    let scheduling_results: Vec<String> = folder
        .map(|path| path.unwrap().path())
        .filter(|path| path.is_file())
        .map(|path| (path.clone(), std::fs::read_to_string(path).unwrap()))
        .map(|(path, content)| (path, parse_psp_lib(&content).unwrap()))
        .map(|(path, psp)| {
            (
                path,
                scheduler(
                    psp,
                    scheduler::SchedulerOptions {
                        number_of_iterations: 1500,
                        max_iter_since_best: 750,
                        tabu_list_size: 100,
                        swap_range: 15,
                        parallel: true,
                        iter_since_best_reset: None,
                    },
                ),
            )
        })
        .map(|(path, os)| (path, os.duration))
        .map(|(path, duration)| format!("{path:?}: {duration}"))
        .collect();

    std::fs::write(benchmark.output, scheduling_results.join("\n"))?;

    Ok(())
}
