use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use psp_lib_parser::parse_psp_lib;
use rcpsp::schedulers::simple::simple_schedule;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");

    for file in ["j3011_2.sm", "j1203_6.sm", "j1201_1.sm"] {
        let contents = std::fs::read_to_string(format!("../examples/{file}")).unwrap();
        let psp = parse_psp_lib(contents.as_str()).unwrap();

        group.bench_function(BenchmarkId::new("simple_schedule", file), |b| {
            b.iter(|| {
                simple_schedule(
                    psp.clone(),
                    rcpsp::schedulers::simple::SimpleScheduleOptions {
                        number_of_iterations: 200,
                        max_iter_since_best: 20,
                        tabu_list_size: 15,
                        swap_range: 10,
                    },
                )
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);