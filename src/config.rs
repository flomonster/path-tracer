use crate::renderer::brdf::BrdfType;
use cgmath::*;
use clap::ArgMatches;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Clone, Debug)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub resolution: Vector2<u32>,
    pub quiet: bool,
    pub bounces: usize,
    pub samples: usize,
    pub brdf_type: BrdfType,
}

impl Config {
    /// Load a config from program arguments
    pub fn load(args: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        // Load default config
        let mut config = Config::default();

        // Load config file
        if let Some(config_path) = args.value_of("config") {
            config.load_config_file(config_path)?;
        }

        // Apply output
        config.output = args.value_of("output").unwrap().into();

        // Apply input
        config.input = args.value_of("INPUT").unwrap().into();

        // Apply other options and parameters
        config.quiet = args.is_present("quiet");

        Ok(config)
    }

    fn load_config_file(&mut self, config_path: &str) -> Result<(), Box<dyn Error>> {
        let docs = YamlLoader::load_from_str(&fs::read_to_string(config_path)?).unwrap();
        let config = docs[0].clone();
        let config = match config.into_hash() {
            Some(config) => config,
            None => return Err(Box::new(ConfigError::InvalidConfig)),
        };

        // Parse resolution
        if let Some(resolution) = config.get(&Yaml::String("resolution".into())) {
            let resolution = resolution.as_str().unwrap();
            let mut res_iter = resolution.split("x");

            // Check resolution validity
            if res_iter.clone().count() != 2 || res_iter.clone().any(|v| v.parse::<u32>().is_err())
            {
                return Err(Box::new(ConfigError::InvalidResolution(
                    resolution.to_string(),
                )));
            }

            // Apply resolution
            self.resolution = Vector2::new(
                res_iter.next().unwrap().parse().unwrap(),
                res_iter.next().unwrap().parse().unwrap(),
            );
        }

        // Apply bounces
        if let Some(bounces) = config.get(&Yaml::String("bounces".into())) {
            if let Some(bounces) = bounces.as_i64() {
                self.bounces = bounces as usize;
            } else {
                return Err(Box::new(ConfigError::InvalidBounces(bounces.clone())));
            }
        }

        // Apply samples
        if let Some(samples) = config.get(&Yaml::String("samples".into())) {
            if let Some(samples) = samples.as_i64() {
                self.samples = samples as usize;
            } else {
                return Err(Box::new(ConfigError::InvalidSamples(samples.clone())));
            }
        }

        // Apply brdf
        if let Some(brdf) = config.get(&Yaml::String("brdf".into())) {
            if let Some(brdf_str) = brdf.as_str() {
                self.brdf_type = match brdf_str.to_uppercase().parse() {
                    Ok(brdf) => brdf,
                    _ => return Err(Box::new(ConfigError::InvalidBRDF(brdf.clone()))),
                };
            } else {
                return Err(Box::new(ConfigError::InvalidBRDF(brdf.clone())));
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resolution: Vector2::new(800, 800),
            input: Default::default(),
            output: Default::default(),
            quiet: false,
            bounces: 2,
            samples: 16,
            brdf_type: BrdfType::CookTorrance,
        }
    }
}

#[derive(Debug)]
enum ConfigError {
    InvalidResolution(String),
    InvalidBounces(Yaml),
    InvalidSamples(Yaml),
    InvalidBRDF(Yaml),
    InvalidConfig,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidResolution(res) => write!(
                f,
                "Invalid resolution: '{}'\nExample of valid resolution: '1920x1080'",
                res
            ),
            ConfigError::InvalidBounces(yaml) => write!(
                f,
                "Invalid bounces: {:?}\nExample of valid number of bounces: 4",
                yaml
            ),
            ConfigError::InvalidSamples(yaml) => write!(
                f,
                "Invalid samples: {:?}\nExample of valid number of samples: 16",
                yaml
            ),
            ConfigError::InvalidBRDF(yaml) => {
                writeln!(f, "Invalid brdf: '{:?}'\nValid brdf are:'", yaml)?;
                for brdf_type in BrdfType::iter() {
                    writeln!(f, "  - {}", brdf_type.to_string())?;
                }
                Ok(())
            }
            ConfigError::InvalidConfig => write!(f, "Invalid config file !"),
        }
    }
}

impl Error for ConfigError {}
