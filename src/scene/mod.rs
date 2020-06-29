mod model;

use model::Model;
use std::error::Error;
use tobj;

pub struct Scene {
    models: Vec<Model>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { models: vec![] }
    }

    pub fn load(path: &str) -> Result<Scene, Box<dyn Error>> {
        let (models, _materials) = tobj::load_obj(path, true)?;
        let mut scene = Scene::new();

        models
            .iter()
            .for_each(|model| scene.models.push(Model::from(model)));

        Ok(scene)
    }
}
