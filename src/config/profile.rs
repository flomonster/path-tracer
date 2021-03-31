use super::Resolution;
use crate::renderer::brdf::BrdfType;
use crate::renderer::tonemap::TonemapType;
use serde::Deserialize;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub resolution: Resolution,
    #[serde(default = "default_bounces")]
    pub bounces: usize,
    #[serde(default = "default_samples")]
    pub samples: usize,
    #[serde(default)]
    pub brdf: BrdfType,
    #[serde(default)]
    pub tonemap: TonemapType,
    #[serde(default)]
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
            bounces: default_bounces(),
            samples: default_samples(),
            brdf: Default::default(),
            tonemap: Default::default(),
            background_color: [0., 0., 0.],
        }
    }
}

fn default_bounces() -> usize {
    2
}

fn default_samples() -> usize {
    16
}
