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
    pub quiet: bool,
    pub bounces: usize,
    pub samples: usize,
}

impl Config {
    /// Load a config from program arguments
    pub fn load(args: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        // Load default config
        let mut config = Config::default();

        // Parse resolution
        let resolution = args.value_of("resolution").unwrap();
        let mut res_iter = resolution.split("x");

        // Check resolution validity
        if res_iter.clone().count() != 2 || res_iter.clone().any(|v| v.parse::<u32>().is_err()) {
            return Err(Box::new(ConfigError::InvalidResolution(
                resolution.to_string(),
            )));
        }

        // Apply resolution
        config.resolution = Vector2::new(
            res_iter.next().unwrap().parse().unwrap(),
            res_iter.next().unwrap().parse().unwrap(),
        );

        // Apply bounces
        if let Ok(bounces) = args.value_of("bounces").unwrap().parse() {
            config.bounces = bounces;
        } else {
            return Err(Box::new(ConfigError::InvalidBounces(
                args.value_of("bounces").unwrap().to_string(),
            )));
        }

        // Apply samples
        if let Ok(samples) = args.value_of("samples").unwrap().parse() {
            config.samples = samples;
        } else {
            return Err(Box::new(ConfigError::InvalidSamples(
                args.value_of("samples").unwrap().to_string(),
            )));
        }

        // Apply other options and parameters
        config.input = args.value_of("INPUT").unwrap().into();
        config.output = args.value_of("OUTPUT").unwrap().into();
        config.quiet = args.is_present("quiet");

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resolution: Vector2::new(854, 480),
            input: Default::default(),
            output: Default::default(),
            quiet: true,
            bounces: 2,
            samples: 16,
        }
    }
}

#[derive(Debug)]
enum ConfigError {
    InvalidResolution(String),
    InvalidBounces(String),
    InvalidSamples(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidResolution(res) => write!(
                f,
                "Invalid resolution: '{}'\nExample of valid resolution: '1920x1080'",
                res
            ),
            ConfigError::InvalidBounces(value) => write!(
                f,
                "Invalid bounces: '{}'\nExample of valid number of bounces: '4'",
                value
            ),
            ConfigError::InvalidSamples(value) => write!(
                f,
                "Invalid samples: '{}'\nExample of valid number of samples: '16'",
                value
            ),
        }
    }
}

impl Error for ConfigError {}
