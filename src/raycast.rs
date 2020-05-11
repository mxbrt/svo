use na::{base::Vector3, geometry::Point3};

pub struct RaycastHit {
    pub color: u32,
    pub normal: Vector3<f32>,
    pub pos: Point3<f32>,
}

pub trait Raycastable {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>) -> Option<RaycastHit>;
}
