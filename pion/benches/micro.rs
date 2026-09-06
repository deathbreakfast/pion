//! Micro-benchmarks for CPU-bound Pion helpers (observed-health derivation, image flavor).
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use pion::{derive_image_flavor, derive_observed_health, ContainerStatusSummary, ObservedHealth};

fn bench_derive_health(c: &mut Criterion) {
    let summary = ContainerStatusSummary {
        running: 10,
        exited: 1,
        unhealthy: 0,
    };
    c.bench_function("derive_observed_health", |b| {
        b.iter(|| {
            let h: ObservedHealth = derive_observed_health(black_box(&summary));
            black_box(h);
        });
    });
}

fn bench_image_flavor(c: &mut Criterion) {
    let samples = [("nginx", "alpine"), ("app", "1.2.3"), ("redis", "7")];
    c.bench_function("derive_image_flavor", |b| {
        b.iter(|| {
            for (name, tag) in &samples {
                black_box(derive_image_flavor(black_box(name), black_box(tag)));
            }
        });
    });
}

criterion_group!(benches, bench_derive_health, bench_image_flavor);
criterion_main!(benches);
