mod cook_torrance;

use super::MaterialSample;
use cgmath::*;
pub use cook_torrance::CookTorrance;
use serde::Deserialize;

pub trait Brdf {
    fn sample(&mut self, geometric_normal: Vector3<f32>, v: Vector3<f32>) -> Vector3<f32>;

    fn eval_direct(
        &self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>,  // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
    ) -> Vector3<f32>;

    fn eval_indirect(
        &self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>,  // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
    ) -> Vector3<f32>;

    fn pdf(&self) -> f32;
}

// Transform any coordinate system to world coordinates
pub fn transform_to_world(vec: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    let nt = if n.x.abs() > n.y.abs() {
        Vector3::new(n.z, 0., -n.x) / (n.x * n.x + n.z * n.z).sqrt()
    } else {
        Vector3::new(0., -n.z, n.y) / (n.y * n.y + n.z * n.z).sqrt()
    };
    let nb = n.cross(nt);

    Vector3::new(
        vec.x * nb.x + vec.y * n.x + vec.z * nt.x,
        vec.x * nb.y + vec.y * n.y + vec.z * nt.y,
        vec.x * nb.z + vec.y * n.z + vec.z * nt.z,
    )
}

#[derive(Copy, Debug, Clone, Deserialize)]
pub enum BrdfType {
    #[serde(rename = "COOK_TORRANCE")]
    CookTorrance,
}

impl Default for BrdfType {
    fn default() -> Self {
        Self::CookTorrance
    }
}

pub fn get_brdf(material_sample: &MaterialSample, brdf_type: BrdfType) -> Box<dyn Brdf> {
    match brdf_type {
        BrdfType::CookTorrance => Box::new(CookTorrance::new(material_sample)),
    }
}
