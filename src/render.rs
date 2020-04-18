use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3};

use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;

pub struct RaycastResult {
    pub color: u32,
    pub normal: u32,
    pub visit_cnt: u32,
}

pub trait Renderable {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>, result: &mut RaycastResult) -> bool;
}

pub struct Renderer {
    width: f32,
    height: f32,
    fov_factor: f32,
    origin: Point3<f32>,
    aspect_ratio: f32,
}

impl Renderer {
    pub fn new(width: usize, height: usize, fov: f32) -> Renderer {
        let mut res = Renderer {
            width: width as f32,
            height: height as f32,
            fov_factor: 0.0,
            origin: Point3::new(0.0, 0.0, 0.0),
            aspect_ratio: 0.0,
        };
        res.resize(width, height, fov);
        res
    }

    pub fn resize(&mut self, width: usize, height: usize, fov: f32) {
        self.width = width as f32;
        self.height = height as f32;
        self.fov_factor = f32::tan(fov.to_radians() / 2.0);
        self.aspect_ratio = self.width / self.height;
    }

    pub fn render(
        &self,
        renderables: &Vec<&(dyn Renderable + Sync)>,
        buffer: &mut [u32],
        view: &Matrix4<f32>,
    ) {
        let pos = view.transform_point(self.origin);
        let iwidth = self.width as usize;
        buffer.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = (i % iwidth) as f32;
            let y = (i / iwidth) as f32;
            let u = (2.0 * (x + 0.5) / self.width - 1.0) * self.aspect_ratio * self.fov_factor;
            let v = (1.0 - 2.0 * (y + 0.5) / self.height) * self.fov_factor;
            let dir = view.transform_vector(Vector3::new(u, v, -1.0)).normalize();

            let mut raycast_result = RaycastResult {
                color: 0,
                normal: 0,
                visit_cnt: 0,
            };
            for renderable in renderables {
                if renderable.raycast(pos, dir, &mut raycast_result) {
                    break;
                }
            }
            *pixel = raycast_result.color;
        });
    }
}
