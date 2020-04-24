use cgmath::{prelude::*, Matrix4, Point3, Vector3};
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;
use specs::{prelude::*, System};

use crate::light::LightComponent;
use crate::raycast::{raycast_scene, RaycastComponent};
use crate::transform::TransformComponent;

#[derive(Default)]
pub struct ScreenBuffer(pub Vec<u32>);

pub struct RenderSystem {
    width: f32,
    height: f32,
    fov_factor: f32,
    aspect_ratio: f32,
}

impl RenderSystem {
    pub fn new(width: usize, height: usize, fov: f32) -> RenderSystem {
        RenderSystem {
            width: width as f32,
            height: height as f32,
            fov_factor: f32::tan(fov.to_radians() / 2.0),
            aspect_ratio: width as f32 / height as f32,
        }
    }
}

pub fn shade(
    color: &Vector3<f32>,
    normal: &Vector3<f32>,
    hit_point: &Point3<f32>,
    scene_objects: &Vec<(&RaycastComponent, &TransformComponent)>,
    lights: &Vec<(&LightComponent, &TransformComponent)>,
) -> u32 {
    let shadow_bias = 0.00001;
    let mut out_color = Vector3::zero();
    for (light, light_transform) in lights {
        let shadow_ray_orig = hit_point + (normal * shadow_bias);
        let shadow_ray_dir = light.direction_from(light_transform, *hit_point);
        let shadow_hit = raycast_scene(scene_objects, shadow_ray_orig, shadow_ray_dir);

        let light_intensity = light.intensity(light_transform, *hit_point);
        if shadow_hit.is_some() {
            let shadow_ray_hit_pos = shadow_hit.unwrap().pos;
            if (shadow_ray_hit_pos - hit_point).magnitude()
                > light.distance(light_transform, *hit_point)
            {
                continue;
            }
        }

        let albedo = 1.0;
        let light_reflected = (albedo / std::f32::consts::PI)
            * f32::max(
                normal.dot(light.direction_from(light_transform, *hit_point)),
                0.0,
            )
            * light_intensity;
        let _color = color.mul_element_wise(light.color * light_reflected);
        out_color = out_color.add_element_wise(_color);
    }
    out_color.x = out_color.x.min(1.0).max(0.0);
    out_color.y = out_color.y.min(1.0).max(0.0);
    out_color.z = out_color.z.min(1.0).max(0.0);
    return (((out_color.x) * 255.0) as u32) << 16
        | (((out_color.y) * 255.0) as u32) << 8
        | (((out_color.z) * 255.0) as u32);
}
#[derive(Clone)]
pub struct ViewMatrix(pub Matrix4<f32>);

impl Default for ViewMatrix {
    fn default() -> ViewMatrix {
        ViewMatrix(Matrix4::zero())
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, RaycastComponent>,
        ReadStorage<'a, TransformComponent>,
        Write<'a, ScreenBuffer>,
        Read<'a, ViewMatrix>,
    );

    fn run(&mut self, (light, raycastable, transform, mut buffer, view): Self::SystemData) {
        let lights = (&light, &transform).join().collect();
        let scene_objects: &Vec<(&RaycastComponent, &TransformComponent)> =
            &(&raycastable, &transform).join().collect();

        let iwidth = self.width as usize;
        let view = view.0.invert().unwrap();
        let origin = view.transform_point(Point3::origin());
        buffer.0.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = (i % iwidth) as f32 + 0.5;
            let y = (i / iwidth) as f32 + 0.5;
            let u = (2.0 * (x + 0.5) / self.width - 1.0) * self.aspect_ratio * self.fov_factor;
            let v = (2.0 * (y + 0.5) / self.height - 1.0) * self.fov_factor;
            let dir = -view.transform_vector(Vector3::new(u, v, 1.0)).normalize();
            let mut color = 0;
            let ray_result = raycast_scene(&scene_objects, origin, dir);
            if ray_result.is_some() {
                let hit = ray_result.unwrap();
                let color_vec = Vector3::new(
                    (hit.color >> 16 & 0xFF) as f32 / 255.0,
                    (hit.color >> 8 & 0xFF) as f32 / 255.0,
                    (hit.color & 0xFF) as f32 / 255.0,
                );
                color = shade(&color_vec, &hit.normal, &hit.pos, &scene_objects, &lights);
            } else {
                let tx = origin.x / dir.x;
                let ty = origin.y / dir.y;
                let tz = origin.z / dir.z;
                let xy_intersect = tx - ty;
                let xz_intersect = tx - tz;
                let yz_intersect = ty - tz;
                if xy_intersect > 0.0000001 && xy_intersect < 0.01 {
                    // z axis: blue
                    color = 0xFF;
                }
                if xz_intersect > 0.0000001 && xz_intersect < 0.01 {
                    // y axis: green
                    color = 0xFF00;
                }
                if yz_intersect > 0.0000001 && yz_intersect < 0.01 {
                    // x axis: red
                    color = 0xFF0000;
                }
            }
            *pixel = color;
        });
    }
}
