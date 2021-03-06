use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use psp_lib_parser::parse_psp_lib;
use rcpsp::scheduler::{custom, rayon, rayon_multi, SchedulerOptions};

struct BenchmarkSet<'a> {
    pub file: &'a str,
    pub config: SchedulerOptions,
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");
    // group.sample_size(50);
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let sets: Vec<BenchmarkSet> = vec![
        BenchmarkSet {
            file: "j3011_2.sm",
            config: SchedulerOptions {
                number_of_iterations: 480,
                max_iter_since_best: 150,
                tabu_list_size: 60,
                swap_range: 30,
                iter_since_best_reset: None,
                parallel: false,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
        BenchmarkSet {
            file: "j3011_2.sm",
            config: SchedulerOptions {
                number_of_iterations: 480,
                max_iter_since_best: 150,
                tabu_list_size: 60,
                swap_range: 30,
                iter_since_best_reset: None,
                parallel: true,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
        BenchmarkSet {
            file: "j1203_6.sm",
            config: SchedulerOptions {
                number_of_iterations: 600,
                max_iter_since_best: 180,
                tabu_list_size: 800,
                swap_range: 60,
                iter_since_best_reset: None,
                parallel: false,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
        BenchmarkSet {
            file: "j1203_6.sm",
            config: SchedulerOptions {
                number_of_iterations: 600,
                max_iter_since_best: 180,
                tabu_list_size: 800,
                swap_range: 60,
                iter_since_best_reset: None,
                parallel: true,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
        BenchmarkSet {
            file: "j1201_1.sm",
            config: SchedulerOptions {
                number_of_iterations: 600,
                max_iter_since_best: 180,
                tabu_list_size: 800,
                swap_range: 60,
                iter_since_best_reset: None,
                parallel: false,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
        BenchmarkSet {
            file: "j1201_1.sm",
            config: SchedulerOptions {
                number_of_iterations: 600,
                max_iter_since_best: 180,
                tabu_list_size: 800,
                swap_range: 60,
                iter_since_best_reset: None,
                parallel: true,
                schedule_count: 10,
                schedule_duration: None,
            },
        },
    ];

    for set in sets {
        let BenchmarkSet { file, config } = set;

        let contents = std::fs::read_to_string(format!("../examples/{file}")).unwrap();
        let psp = parse_psp_lib(contents.as_str()).unwrap();

        group.bench_with_input(
            BenchmarkId::new(
                format!(
                    "rayon_{}",
                    if config.parallel {
                        "parallel"
                    } else {
                        "single"
                    }
                ),
                format!("{file}/{}", config.number_of_iterations),
            ),
            &config,
            |b, config| b.iter(|| rayon::scheduler(psp.clone(), config.clone())),
        );
        group.bench_with_input(
            BenchmarkId::new(
                format!(
                    "custom_{}",
                    if config.parallel {
                        "parallel"
                    } else {
                        "single"
                    }
                ),
                format!("{file}/{}", config.number_of_iterations),
            ),
            &config,
            |b, config| b.iter(|| custom::scheduler(psp.clone(), config.clone())),
        );
        group.bench_with_input(
            BenchmarkId::new(
                format!(
                    "rayon_multi_{}",
                    if config.parallel {
                        "parallel"
                    } else {
                        "single"
                    }
                ),
                format!("{file}/{}", config.number_of_iterations),
            ),
            &config,
            |b, config| b.iter(|| rayon_multi::scheduler(psp.clone(), config.clone())),
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
