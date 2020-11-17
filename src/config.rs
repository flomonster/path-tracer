use cgmath::*;
use clap::ArgMatches;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub resolution: Vector2<u32>,
}

impl Config {
    /// Load a config from program arguments
    pub fn load(args: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let resolution = args.value_of("resolution").unwrap();
        let mut res_iter = resolution.split("x");

        // Check resolution validity
        if res_iter.clone().count() != 2 || res_iter.clone().any(|v| v.parse::<u32>().is_err()) {
            return Err(Box::new(ConfigError::InvalidResolution(
                resolution.to_string(),
            )));
        }

        // Parse resolution
        let resolution = Vector2::new(
            res_iter.next().unwrap().parse().unwrap(),
            res_iter.next().unwrap().parse().unwrap(),
        );

        // Build config
        Ok(Config {
            input: args.value_of("INPUT").unwrap().into(),
            output: args.value_of("OUTPUT").unwrap().into(),
            resolution,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resolution: Vector2::new(854, 480),
            input: Default::default(),
            output: Default::default(),
        }
    }
}

#[derive(Debug)]
enum ConfigError {
    InvalidResolution(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidResolution(res) => write!(
                f,
                "Invalid resolution: '{}'\nExample of valid resolution: '1920x1080'",
                res
            ),
        }
    }
}

impl Error for ConfigError {}
