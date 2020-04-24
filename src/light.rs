use cgmath::{prelude::*, Point3, Vector3};
use specs::{prelude::*, Component};

use crate::transform::TransformComponent;

pub enum LightType {
    SphericalLight,
    DirectionalLight,
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub intensity: f32,
    pub color: Vector3<f32>,
}

impl LightComponent {
    pub fn intensity(&self, transform: &TransformComponent, point: Point3<f32>) -> f32 {
        match self.light_type {
            LightType::DirectionalLight => self.intensity,
            LightType::SphericalLight => {
                let r2 = (transform.position - point).magnitude2();
                self.intensity / (4.0 * std::f32::consts::PI * r2)
            }
        }
    }

    pub fn direction_from(
        &self,
        transform: &TransformComponent,
        point: Point3<f32>,
    ) -> Vector3<f32> {
        match self.light_type {
            LightType::DirectionalLight => {
                -(transform.rotation * Vector3::<f32>::new(1.0, 1.0, 1.0)).normalize()
            }
            LightType::SphericalLight => (transform.position - point).normalize(),
        }
    }

    pub fn distance(&self, transform: &TransformComponent, point: Point3<f32>) -> f32 {
        match self.light_type {
            LightType::DirectionalLight => std::f32::INFINITY,
            LightType::SphericalLight => (transform.position - point).magnitude(),
        }
    }
}
