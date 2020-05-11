use na::{base::Matrix4, geometry::Point4};

pub struct Camera {
    pub position: Point4<f32>,
    pub rotation: Matrix4<f32>,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point4::origin(),
            rotation: Matrix4::identity(),
            fov: 60.0,
        }
    }
}
