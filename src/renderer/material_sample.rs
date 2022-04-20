use cgmath::*;

use crate::scene::internal::Material;
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Default)]
pub struct MaterialSample {
    pub metalness: f32,
    pub roughness: f32,
    #[derivative(Default(value = "Zero::zero()"))]
    pub albedo: Vector3<f32>,
    pub opacity: f32,
    #[derivative(Default(value = "Zero::zero()"))]
    pub emissive: Vector3<f32>,
    pub ior: f32,
}

impl MaterialSample {
    pub fn new(material: &Material, tex_coords: &Vector2<f32>) -> Self {
        Self {
            metalness: material.get_metalness(tex_coords),
            roughness: material.get_roughness(tex_coords).max(0.0001), // roughness = 0 breaks the maths (in NDF function)
            albedo: material.get_albedo(tex_coords),
            opacity: material.get_opacity(tex_coords),
            emissive: material.get_emissive(tex_coords),
            ior: material.ior,
        }
    }

    pub fn simple(material: &Material) -> Self {
        Self {
            metalness: material.get_simple_metalness(),
            roughness: material.get_simple_roughness().max(0.0001), // roughness = 0 breaks the maths (in NDF function)
            albedo: material.get_simple_albedo(),
            opacity: material.get_simple_opacity(),
            emissive: material.get_simple_emissive(),
            ior: material.ior,
        }
    }
}
