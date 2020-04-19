use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

pub trait Light {
    fn intensity(&self, hit_point: Point3<f32>) -> f32;
    fn direction_from(&self, hit_point: Point3<f32>) -> Vector3<f32>;
    fn color(&self) -> Vector3<f32>;
    fn distance(&self, hit_point: Point3<f32>) -> f32;
}

pub struct DirectionalLight {
    pub direction: Vector3<f32>,
    pub intensity: f32,
    pub color: Vector3<f32>,
}

pub struct SphericalLight {
    pub position: Point3<f32>,
    pub intensity: f32,
    pub color: Vector3<f32>,
}

impl Light for SphericalLight {
    fn intensity(&self, hit_point: Point3<f32>) -> f32 {
        let r2 = (self.position - hit_point).magnitude2();
        self.intensity / (4.0 * std::f32::consts::PI * r2)
    }

    fn direction_from(&self, hit_point: Point3<f32>) -> Vector3<f32> {
        (self.position - hit_point).normalize()
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }

    fn distance(&self, hit_point: Point3<f32>) -> f32 {
        (self.position - hit_point).magnitude()
    }
}

impl Light for DirectionalLight {
    fn intensity(&self, _: Point3<f32>) -> f32 {
        self.intensity
    }

    fn direction_from(&self, _: Point3<f32>) -> Vector3<f32> {
        -self.direction.normalize()
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }

    fn distance(&self, _: Point3<f32>) -> f32 {
        std::f32::INFINITY
    }
}
