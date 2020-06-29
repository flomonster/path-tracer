mod model;

use model::Model;
use tobj;
use std::error::Error;
use std::io;

pub struct Scene {
    models: Vec<Model>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            models: vec![]
        }
    }

    pub fn load(path: &str) -> Result<Scene, Box<dyn Error>> {
        // open file
        let file = std::fs::File::open(path)?;
        let file_reader = io::BufReader::new(file);
        
        let (models, materials) = tobj::load_obj(path, true)?;
        let mut scene = Scene::new();

        models.iter().for_each(|model| scene.models.push(Model::from(model)));


        Ok(scene)
    }
}
