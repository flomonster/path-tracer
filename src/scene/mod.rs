mod gltf;
pub mod internal;
mod isf;

pub use gltf::convert_gltf_to_isf;

use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use internal::Scene;

pub fn load_internal<P: AsRef<Path>>(path: P) -> Result<Scene, Box<dyn Error + Send + Sync>> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let isf_scene = serde_json::from_reader(reader)?;
    let root_path = PathBuf::from(path.as_ref()).parent().unwrap().to_path_buf();
    Ok(Scene::load(isf_scene, root_path))
}
