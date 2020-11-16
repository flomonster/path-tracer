pub mod model;

use easy_gltf::{Camera, Light};

use model::Model;
use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub models: Vec<Model>,
    pub camera: Camera,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn load(config: &crate::Config) -> Result<Scene, Box<dyn Error>> {
        let mut scene = Self::default();
        let scenes = easy_gltf::load(&config.input)?;

        if scenes.is_empty() {
            // TODO: Return error instead
            panic!("No scene found")
        }

        for model in scenes[0].models.iter() {
            scene.models.push(model.into());
        }

        if scenes[0].cameras.is_empty() {
            // TODO: Return error instead
            panic!("No camera found")
        }
        scene.camera = scenes[0].cameras[0].clone();

        scene.lights = scenes[0].lights.clone();
        Ok(scene)
    }
}
