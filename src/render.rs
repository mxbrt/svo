use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3};

use crate::light::Light;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;

pub struct RaycastResult {
    pub color: u32,
    pub normal: Vector3<f32>,
    pub visit_cnt: u32,
    pub hit_pos: Point3<f32>,
}

pub trait Renderable {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>, result: &mut RaycastResult) -> bool;
    fn get_model(&self) -> &Matrix4<f32>;
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

    pub fn raytrace_scene(
        &self,
        renderables: &Vec<&(dyn Renderable + Sync)>,
        orig: Point3<f32>,
        dir: Vector3<f32>,
    ) -> Option<RaycastResult> {
        let mut raycast_result = RaycastResult {
            color: 0,
            normal: Vector3::zero(),
            visit_cnt: 0,
            hit_pos: Point3::origin(),
        };
        for renderable in renderables {
            //let model = renderable.get_model();
            if renderable.raycast(orig, dir, &mut raycast_result) {
                return Some(raycast_result);
            }
        }
        None
    }

    pub fn shade(
        &self,
        color: &Vector3<f32>,
        normal: &Vector3<f32>,
        hit_point: &Point3<f32>,
        renderables: &Vec<&(dyn Renderable + Sync)>,
        lights: &Vec<&(dyn Light + Sync)>,
    ) -> u32 {
        let shadow_bias = 0.000001;
        let mut out_color = Vector3::zero();
        for light in lights {
            let shadow_ray_orig = hit_point + (normal * shadow_bias);
            let shadow_ray_dir = light.direction_from(*hit_point);
            let ray_result = self.raytrace_scene(renderables, shadow_ray_orig, shadow_ray_dir);

            let light_intensity = light.intensity(*hit_point);
            if ray_result.is_some() {
                let shadow_ray_hit_pos = ray_result.unwrap().hit_pos;
                if (shadow_ray_hit_pos - hit_point).magnitude() > light.distance(*hit_point) {
                    continue;
                }
            }

            let albedo = 1.0;
            let light_reflected = (albedo / std::f32::consts::PI)
                * f32::max(normal.dot(light.direction_from(*hit_point)), 0.0)
                * light_intensity;
            let _color = color.mul_element_wise(light.color() * light_reflected);
            out_color = out_color.add_element_wise(_color);
        }
        out_color.x = out_color.x.min(1.0).max(0.0);
        out_color.y = out_color.y.min(1.0).max(0.0);
        out_color.z = out_color.z.min(1.0).max(0.0);
        return (((out_color.x) * 255.0) as u32) << 16
            | (((out_color.y) * 255.0) as u32) << 8
            | (((out_color.z) * 255.0) as u32);
    }

    pub fn render(
        &self,
        renderables: &Vec<&(dyn Renderable + Sync)>,
        lights: &Vec<&(dyn Light + Sync)>,
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
            let dir = view.transform_vector(Vector3::new(u, v, 1.0)).normalize();

            let mut color = 0;
            let ray_result = self.raytrace_scene(renderables, pos, dir);
            if ray_result.is_some() {
                let hit = ray_result.unwrap();
                color = self.shade(
                    &Vector3::new(
                        (hit.color >> 16 & 0xFF) as f32 / 255.0,
                        (hit.color >> 8 & 0xFF) as f32 / 255.0,
                        (hit.color & 0xFF) as f32 / 255.0,
                    ),
                    &hit.normal,
                    &hit.hit_pos,
                    renderables,
                    lights,
                );
            }
            *pixel = color;
        });
    }
}
