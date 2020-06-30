mod camera;
pub mod model;

pub use camera::Camera;
use model::Model;
use std::error::Error;
use std::path::PathBuf;
use tobj;

pub struct Scene {
    pub models: Vec<Model>,
    pub camera: Camera,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            models: vec![],
            camera: Camera::new(),
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
