extern crate nalgebra as na;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simba::simd::f32x8;

use rand::Rng;
use svo::simd;

fn random_f32x8() -> f32x8 {
    rand::thread_rng().gen::<[f32; 8]>().into()
}

fn random_vector() -> na::Vector4<f32x8> {
    [
        random_f32x8(),
        random_f32x8(),
        random_f32x8(),
        random_f32x8(),
    ]
    .into()
}

fn random_vectorlist(n: u64) -> simd::VectorList {
    simd::VectorList {
        0: (0..n / 8).map(|_| random_vector()).collect(),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let n = 1_000_000;
    let vectors = random_vectorlist(n);
    let m = na::Matrix4::from_fn(|_, _| rand::thread_rng().gen());
    c.bench_function("vectorlist 10^5", |b| {
        b.iter(|| black_box(vectors.transform(m)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
