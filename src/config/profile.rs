use super::Resolution;
use crate::renderer::brdf::BrdfType;
use crate::renderer::tonemap::TonemapType;
use derivative::Derivative;
use serde::Deserialize;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Copy, Clone, Debug, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Profile {
    #[serde(default)]
    pub resolution: Resolution,
    #[derivative(Default(value = "default_bounces()"))]
    #[serde(default = "default_bounces")]
    pub bounces: usize,
    #[derivative(Default(value = "default_samples()"))]
    #[serde(default = "default_samples")]
    pub samples: usize,
    #[serde(default)]
    pub brdf: BrdfType,
    #[serde(default)]
    pub tonemap: TonemapType,
    #[serde(default)]
    pub nb_threads: usize,
}

impl Profile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let serialized = &read_to_string(path)?;
        Ok(serde_yaml::from_str(serialized)?)
    }
}

fn default_bounces() -> usize {
    4
}

fn default_samples() -> usize {
    64
}
