use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use psp_lib_parser::parse_psp_lib;
use rcpsp::scheduler;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");
    group.sample_size(50);
    group.sampling_mode(criterion::SamplingMode::Flat);

    for file in ["j3011_2.sm", "j1203_6.sm", "j1201_1.sm"] {
        let contents = std::fs::read_to_string(format!("../examples/{file}")).unwrap();
        let psp = parse_psp_lib(contents.as_str()).unwrap();

        let configs = vec![
            rcpsp::scheduler::SchedulerOptions {
                number_of_iterations: 200,
                max_iter_since_best: 20,
                tabu_list_size: 15,
                swap_range: 10,
                parallel: false,
            },
            rcpsp::scheduler::SchedulerOptions {
                number_of_iterations: 1000,
                max_iter_since_best: 100,
                tabu_list_size: 25,
                swap_range: 15,
                parallel: false,
            },
            rcpsp::scheduler::SchedulerOptions {
                number_of_iterations: 200,
                max_iter_since_best: 20,
                tabu_list_size: 15,
                swap_range: 10,
                parallel: true,
            },
            rcpsp::scheduler::SchedulerOptions {
                number_of_iterations: 1000,
                max_iter_since_best: 100,
                tabu_list_size: 25,
                swap_range: 15,
                parallel: true,
            },
        ];

        for config in configs {
            group.bench_with_input(
                BenchmarkId::new(
                    format!(
                        "scheduler_{}",
                        if config.parallel {
                            "parallel"
                        } else {
                            "single"
                        }
                    ),
                    format!("{file}/{}", config.number_of_iterations),
                ),
                &config,
                |b, config| b.iter(|| scheduler(psp.clone(), config.clone())),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
