mod camera;
mod light;
pub mod model;

pub use camera::Camera;
pub use light::Light;

use model::Model;
use std::error::Error;
use tobj;

pub struct Scene {
    pub models: Vec<Model>,
    pub camera: Camera,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            models: vec![],
            camera: Camera::new(),
            lights: vec![],
        }
    }

    pub fn load(config: &crate::Config) -> Result<Scene, Box<dyn Error>> {
        let (models, materials) = tobj::load_obj(&config.input, true)?;

        // Get base path needed to retrieve texture materials
        let mut path = config.input.clone();
        path.pop();

        let mut scene = Scene::new();

        models
            .iter()
            .for_each(|model| scene.models.push(Model::load(model, &materials, &path)));

        Ok(scene)
    }
}
