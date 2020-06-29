mod model;

use model::Model;
use std::error::Error;
use std::path::PathBuf;
use tobj;

pub struct Scene {
    models: Vec<Model>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { models: vec![] }
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
