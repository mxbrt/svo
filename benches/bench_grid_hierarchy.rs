extern crate nalgebra as na;

use std::f32::consts::PI;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use svo::{grid, simd, voxel_csv};

fn criterion_benchmark(c: &mut Criterion) {
    let voxels = voxel_csv::read(&"data/torus_128.csv".into());
    let mut grid = grid::Hierarchy::new();
    let mut simd_list = simd::VectorList::from(&voxels);
    let m = na::Matrix4::from_euler_angles(PI / 15.0, PI / 20.0, 0.0)
        .append_translation(&na::Vector3::from([256.0; 3]));
    c.bench_function("rasterizer", |b| {
        b.iter(|| black_box(grid.rasterize(&mut simd_list, &m)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
