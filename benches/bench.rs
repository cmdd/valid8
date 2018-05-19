#[macro_use]
extern crate criterion;
extern crate valid8;

use criterion::{Criterion, Fun};
use valid8::*;

fn validate_ascii(c: &mut Criterion) {
    let manual = Fun::new("std::simd", |b, i| b.iter(|| ascii::std(*i)));
    let faster = Fun::new("faster", |b, i| b.iter(|| ascii::faster(*i)));
    let suparch = Fun::new("super_arch", |b, i| b.iter(|| ascii::super_arch(*i)));
    let arch = Fun::new("arch", |b, i| b.iter(|| ascii::arch(*i)));
    let default = Fun::new("default", |b, i: &&[u8]| b.iter(|| (*i).is_ascii()));

    let s = include_bytes!("data/ascii1");

    let fs = vec![manual, faster, arch, suparch, default];
    c.bench_functions("validate_ascii", fs, s);
}

criterion_group!(benches, validate_ascii);
criterion_main!(benches);
