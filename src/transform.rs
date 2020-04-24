use cgmath::{Matrix4, Point3, Quaternion, Vector3};
use specs::{prelude::*, Component};

#[derive(Component)]
pub struct TransformComponent {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub inv_model: Matrix4<f32>,
}
