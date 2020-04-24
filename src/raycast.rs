use cgmath::{prelude::*, Point3, Vector3};
use specs::{prelude::*, Component};

use crate::transform::TransformComponent;

#[derive(Component)]
pub struct RaycastComponent(pub std::sync::Arc<dyn Raycastable + Sync + Send>);

pub struct RaycastHit {
    pub color: u32,
    pub normal: Vector3<f32>,
    pub pos: Point3<f32>,
}

pub trait Raycastable {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>) -> Option<RaycastHit>;
}

pub fn raycast_scene(
    scene_objects: &Vec<(&RaycastComponent, &TransformComponent)>,
    origin: Point3<f32>,
    dir: Vector3<f32>,
) -> Option<RaycastHit> {
    for (raycastable, transform) in scene_objects {
        let dir = transform.inv_model.transform_vector(dir);
        let origin = transform.inv_model.transform_point(origin);
        let ray_result = raycastable.0.raycast(origin, dir);
        if ray_result.is_some() {
            let mut hit = ray_result.unwrap();
            // TODO: store model matrix
            let model = transform.inv_model.invert().unwrap();
            hit.normal = model.transform_vector(hit.normal);
            hit.pos = model.transform_point(hit.pos);
            return Some(hit);
        }
    }
    None
}
