pub mod model;

use easy_gltf::{Camera, Light};

use kdtree_ray::*;
use model::Model;
use rayon::prelude::*;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Scene {
    pub models: KDtree<Model>,
    pub camera: Camera,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn load(config: &crate::Config) -> Result<Scene, Box<dyn Error>> {
        let scenes = easy_gltf::load(&config.input)?;

        if scenes.is_empty() {
            // TODO: Return error instead
            panic!("No scene found")
        }

        let models = KDtree::new(scenes[0].models.par_iter().map(|m| m.into()).collect());

        if scenes[0].cameras.is_empty() {
            // TODO: Return error instead
            panic!("No camera found")
        }
        let camera = scenes[0].cameras[0].clone();

        let lights = scenes[0].lights.clone();

        Ok(Scene {
            models,
            camera,
            lights,
        })
    }
}
