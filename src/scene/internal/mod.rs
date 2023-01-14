mod camera;
mod light;
mod material;
mod model;
mod texture_bank;
mod triangle;
mod vertex;

use std::path::PathBuf;

use cgmath::Vector3;
use kdtree_ray::KDTree;

pub use camera::Camera;
pub use light::Light;
pub use material::Material;
pub use model::Model;
pub use triangle::Triangle;
pub use vertex::Vertex;

use self::texture_bank::TextureBank;

use super::isf;

#[derive(Debug, Clone)]
pub struct Scene {
    pub models: Vec<Model>,
    pub kdtree: KDTree,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub background: Vector3<f32>,
}

impl Scene {
    pub fn load(isf: isf::Scene, root_path: PathBuf) -> Self {
        let mut texture_bank = TextureBank::new(root_path);
        let models = isf
            .models
            .into_iter()
            .map(|m| Model::load(m, &mut texture_bank))
            .collect();
        let kdtree = KDTree::build(&models);

        Self {
            kdtree,
            models,
            camera: isf.camera.into(),
            lights: isf.lights.into_iter().map(|l| l.into()).collect(),
            background: isf.background.into(),
        }
    }
}
