use cgmath::*;
use easy_gltf::Material;
use std::sync::Arc;

pub struct MaterialSample {
    pub metalness: f32,
    pub roughness: f32,
    pub albedo: Vector3<f32>,
    pub ambient_occlusion: Option<f32>,
}

impl MaterialSample {
    pub fn new(material: &Arc<Material>, tex_coords: Vector2<f32>) -> Self {
        Self {
            metalness: material.get_metallic(tex_coords),
            roughness: material.get_roughness(tex_coords).max(0.0001), // roughness = 0 breaks the maths (in NDF function)
            albedo: material.get_base_color(tex_coords),
            ambient_occlusion: material.get_occlusion(tex_coords),
        }
    }
}

impl Default for MaterialSample {
    fn default() -> Self {
        Self {
            metalness: 0.,
            roughness: 0.,
            albedo: Zero::zero(),
            ambient_occlusion: None,
        }
    }
}
