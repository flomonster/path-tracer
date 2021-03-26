use super::Resolution;
use crate::renderer::brdf::BrdfType;
use crate::renderer::tonemap::TonemapType;
use serde::Deserialize;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Profile {
    pub resolution: Resolution,
    pub bounces: usize,
    pub samples: usize,
    pub brdf: BrdfType,
    pub tonemap: TonemapType,
    pub background_color: [f32; 3],
}

impl Profile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let serialized = &read_to_string(path)?;
        Ok(serde_yaml::from_str(serialized)?)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            resolution: Default::default(),
            bounces: 2,
            samples: 16,
            brdf: BrdfType::CookTorrance,
            tonemap: TonemapType::Filmic,
            background_color: [0., 0., 0.],
        }
    }
}
