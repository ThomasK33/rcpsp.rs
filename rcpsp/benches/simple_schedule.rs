/*
#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use psp_lib_parser::parse_psp_lib;
use rcpsp::schedulers::simple::simple_schedule;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");

    for file in ["j3011_2.sm", "j1203_6.sm", "j1201_1.sm"] {
        let contents = std::fs::read_to_string(format!("../examples/{file}")).unwrap();
        let psp = parse_psp_lib(contents.as_str()).unwrap();

        group.bench_with_input(
            BenchmarkId::new("simple_schedule", file),
            &file,
            |b, &file| b.iter(|| simple_schedule(psp.clone(), 800, 10)),
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
*/