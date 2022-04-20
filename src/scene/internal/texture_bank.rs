use std::{collections::HashMap, path::PathBuf, sync::Arc};

use image::{GrayImage, RgbImage};

#[derive(Debug)]
pub struct TextureBank {
    pub root_path: PathBuf,
    pub rgb_textures: HashMap<String, Arc<RgbImage>>,
    pub gray_textures: HashMap<String, Arc<GrayImage>>,
}

impl TextureBank {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            rgb_textures: Default::default(),
            gray_textures: Default::default(),
        }
    }

    pub fn get_rgb(&mut self, path: String) -> Arc<RgbImage> {
        let path: String = self
            .root_path
            .join(path)
            .canonicalize()
            .expect("Invalid path")
            .to_str()
            .unwrap()
            .into();

        self.rgb_textures
            .entry(path.clone())
            .or_insert_with(|| Arc::new(image::open(&path).unwrap().into_rgb8()))
            .clone()
    }

    pub fn get_gray(&mut self, path: String) -> Arc<GrayImage> {
        let path: String = self
            .root_path
            .join(path)
            .canonicalize()
            .expect("Invalid path")
            .to_str()
            .unwrap()
            .into();

        self.gray_textures
            .entry(path.clone())
            .or_insert_with(|| Arc::new(image::open(&path).unwrap().into_luma8()))
            .clone()
    }
}
