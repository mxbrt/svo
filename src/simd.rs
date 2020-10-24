extern crate nalgebra as na;
use simba::simd::f32x8;

use crate::morton;
use crate::voxel;

pub struct VectorList(pub Vec<na::Vector4<f32x8>>);

pub const N_LANES: usize = 8;
impl VectorList {
    pub fn _print_stats(&self) {
        let len = self.0.len();
        let size = std::mem::size_of::<na::Vector4<f32x8>>() * len;
        println!(
            "simd vecs: {} <-> points: ~{}, size: {} kB",
            len,
            len * N_LANES,
            size / 1024
        );
    }

    pub fn transform(&self, m: na::Matrix4<f32>) -> Self {
        let mut points = self.0.clone();
        let mut simd_matrix: na::Matrix4<f32x8> = na::Matrix4::zeros();

        for i in 0..16 {
            simd_matrix[i] = [m[i]; N_LANES].into();
        }

        for i in 0..self.0.len() {
            points[i] = simd_matrix * self.0[i];
        }

        Self { 0: points }
    }
}

impl From<&voxel::Grid> for VectorList {
    fn from(grid: &voxel::Grid) -> Self {
        let mut simd_vectors = Vec::new();
        let mut xs = [0.0f32; N_LANES];
        let mut ys = [0.0f32; N_LANES];
        let mut zs = [0.0f32; N_LANES];
        let ws = [1.0f32; N_LANES];
        let mut idx = 0;
        let grid_volume = grid.size.pow(3) as u64;
        for m in 0..grid_volume {
            let (x, y, z) = morton::decode_3d(m);
            if grid.data[x as usize][y as usize][z as usize] {
                xs[idx] = x as f32;
                ys[idx] = y as f32;
                zs[idx] = z as f32;
                idx += 1;
            }
            if idx == N_LANES || (idx > 0 && m == grid_volume - 1) {
                simd_vectors.push([xs.into(), ys.into(), zs.into(), ws.into()].into());
                idx = 0;
                xs = [0.0f32; N_LANES];
                ys = [0.0f32; N_LANES];
                zs = [0.0f32; N_LANES];
            }
        }
        Self { 0: simd_vectors }
    }
}

impl From<&Vec<na::Vector4<f32>>> for VectorList {
    fn from(vectors: &Vec<na::Vector4<f32>>) -> Self {
        let mut simd_vectors = Vec::new();
        assert!(vectors.len() % N_LANES == 0);
        for i in (0..vectors.len()).step_by(N_LANES) {
            let mut xs = [0.0f32; N_LANES];
            let mut ys = [0.0f32; N_LANES];
            let mut zs = [0.0f32; N_LANES];
            let mut ws = [0.0f32; N_LANES];
            for j in 0..N_LANES {
                let u = vectors[i + j];
                xs[j] = u.x;
                ys[j] = u.y;
                zs[j] = u.z;
                ws[j] = u.w;
            }
            simd_vectors.push([xs.into(), ys.into(), zs.into(), ws.into()].into());
        }
        Self { 0: simd_vectors }
    }
}

#[cfg(test)]
mod tests {
    extern crate nalgebra as na;
    use crate::simd;
    use rand::Rng;

    fn random_vector() -> na::Vector4<f32> {
        rand::thread_rng().gen::<[f32; 4]>().into()
    }

    #[test]
    fn compare_scalar_simd_transform() {
        let n = 1024;
        let vectors: Vec<na::Vector4<f32>> = (0..n).map(|_| random_vector()).collect();
        let m = na::Matrix4::from_fn(|_, _| rand::thread_rng().gen());

        let simd_vectorlist = simd::VectorList::from(&vectors);
        let simd_transformed = simd_vectorlist.transform(m);
        let scalar_transformed: Vec<na::Vector4<f32>> = vectors.iter().map(|x| m * x).collect();

        let mut scalar_idx = 0;
        for simd_vector in simd_transformed.0 {
            let xs: [f32; simd::N_LANES] = simd_vector[0].into();
            let ys: [f32; simd::N_LANES] = simd_vector[1].into();
            let zs: [f32; simd::N_LANES] = simd_vector[2].into();
            let ws: [f32; simd::N_LANES] = simd_vector[3].into();
            for i in 0..simd::N_LANES {
                let simd_scalar = na::Vector4::from([xs[i], ys[i], zs[i], ws[i]]);
                assert_eq!(scalar_transformed[scalar_idx], simd_scalar);
                scalar_idx += 1;
            }
        }
    }
}
