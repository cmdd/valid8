#[macro_use]
extern crate criterion;
extern crate valid8;

use criterion::{Criterion, Fun, ParameterizedBenchmark, Throughput};
use valid8::*;

fn validate_ascii(c: &mut Criterion) {
    let s1 = include_bytes!("data/ascii1");
    let params = vec![s1];

    let fs = vec![manual, faster, arch, suparch, default];
    c.bench(
        "ascii",
        fs,
        ParameterizedBenchmark::new("validate", |b, i| b.iter(|| ascii::validate(*i)), params)
            .with_function("faster", |b, i| b.iter(|| ascii::faster(*i)))
            .with_function("super_arch", |b, i| b.iter(|| ascii::super_arch(*i)))
            .with_function("arch", |b, i| b.iter(|| ascii::arch(*i)))
            .with_function("default", |b, i: &&[u8]| b.iter(|| (*i).is_ascii()))
            .throughput(|s| Throughput::Bytes(s.len() as u32)),
    );
}

criterion_group!(benches, validate_ascii);
criterion_main!(benches);
