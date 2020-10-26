extern crate nalgebra as na;
use simba::simd::f32x8;

use crate::{morton, simd, sparse};

const BRICK_WIDTH: usize = 16;

pub struct Hierarchy<'a> {
    levels: Vec<sparse::Slice<'a, u8>>,
}

impl<'a> Hierarchy<'a> {
    pub fn new() -> Self {
        let mut levels = Vec::new();
        for level in 1..4 {
            let level_volume = BRICK_WIDTH.pow(3).pow(level as u32);
            levels.push(sparse::Slice::new(level_volume));
        }
        Self { levels }
    }

    pub fn rasterize(&mut self, voxel_list: &mut simd::VectorList, transform: &na::Matrix4<f32>) {
        let mut simd_matrix: na::Matrix4<f32x8> = na::Matrix4::zeros();

        for i in 0..16 {
            simd_matrix[i] = [transform[i]; simd::N_LANES].into();
        }

        for simd_vector in &voxel_list.0 {
            let mut transformed_simd_vector = simd_matrix * simd_vector;
            transformed_simd_vector /= transformed_simd_vector.w;

            // TODO: use simd to round vector
            let xs: [f32; simd::N_LANES] = transformed_simd_vector[0].into();
            let ys: [f32; simd::N_LANES] = transformed_simd_vector[1].into();
            let zs: [f32; simd::N_LANES] = transformed_simd_vector[2].into();
            for i in 0..simd::N_LANES {
                let (x, y, z) = (
                    xs[i].round() as u64,
                    ys[i].round() as u64,
                    zs[i].round() as u64,
                );
                let m = morton::encode_3d(x, y, z) as usize;
                self.levels[2].0[m] = 1;
                self.levels[1].0[m >> 12] = 1;
                self.levels[0].0[m >> 24] = 1;
            }
        }
    }
}
