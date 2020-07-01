mod camera;
mod light;
pub mod model;

pub use camera::Camera;
pub use light::Light;
use model::Model;
use std::error::Error;
use std::path::PathBuf;
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

    pub fn load(path: &str) -> Result<Scene, Box<dyn Error>> {
        let mut path = PathBuf::from(path);
        let (models, materials) = tobj::load_obj(&path, true)?;

        // Get base path needed to retrieve texture materials
        path.pop();

        let mut scene = Scene::new();

        models
            .iter()
            .for_each(|model| scene.models.push(Model::load(model, &materials, &path)));

        Ok(scene)
    }
}
