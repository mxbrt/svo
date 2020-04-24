use cgmath::{Deg, Matrix4, Point3};

pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Matrix4<f32>,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, -3.0),
            rotation: Matrix4::from_angle_y(Deg(0.0)),
            fov: 60.0,
        }
    }
}
