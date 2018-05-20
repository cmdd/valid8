#[macro_use]
extern crate criterion;
extern crate valid8;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};
use valid8::*;
use std::{fmt, ops::Deref};

struct ByteFile(&'static str, &'static [u8]);

impl fmt::Debug for ByteFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ByteFile {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

fn validate_ascii(c: &mut Criterion) {
    let short = ByteFile("valid_short", include_bytes!("data/ascii_short"));
    let medium = ByteFile("valid_medium", include_bytes!("data/ascii_medium"));
    let long = ByteFile("valid_long", include_bytes!("data/ascii_long"));
    let params = vec![short, medium, long];

    c.bench(
        "ascii",
        ParameterizedBenchmark::new("validate", |b, i| b.iter(|| ascii::validate(i)), params)
            .with_function("faster", |b, i| b.iter(|| ascii::faster(i)))
            .with_function("arch", |b, i| b.iter(|| ascii::arch(i)))
            .with_function("super_arch", |b, i| b.iter(|| ascii::super_arch(i)))
            .with_function("default", |b, i| b.iter(|| i.is_ascii()))
            .throughput(|s| Throughput::Bytes(s.len() as u32)),
    );
}

criterion_group!(benches, validate_ascii);
criterion_main!(benches);
