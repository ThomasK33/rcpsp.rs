use anyhow::Result;
use psp_lib_parser::parse_psp_lib;
use rcpsp::scheduler;
use std::time::Instant;

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
                Instant::now(),
                scheduler(
                    psp,
                    scheduler::SchedulerOptions {
                        number_of_iterations: benchmark.number_of_iterations,
                        max_iter_since_best: benchmark.max_iter_since_best,
                        tabu_list_size: benchmark.tabu_list_size,
                        swap_range: benchmark.swap_range,
                        parallel: benchmark.parallel,
                        iter_since_best_reset: benchmark.iter_since_best_reset,
                    },
                ),
            )
        })
        .map(|(path, start_time, os)| {
            let os_duration = os.duration;
            let elapsed = start_time.elapsed();

            format!("{path:?}, {os_duration}, {elapsed:?}")
        })
        .collect();

    std::fs::write(benchmark.output, scheduling_results.join("\n"))?;

    Ok(())
}
