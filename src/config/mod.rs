mod profile;
mod resolution;

use clap::ArgMatches;
pub use profile::Profile;
pub use resolution::Resolution;
use std::error::Error;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub quiet: bool,
    pub profile: Profile,
}

impl Config {
    /// Load a config from program arguments
    pub fn load(args: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        // Load default config
        let mut config = Config::default();

        // Load config file
        if let Some(path) = args.value_of("profile") {
            config.profile = Profile::load(path)?;
        }

        // Apply output
        config.output = args.value_of("output").unwrap().into();

        // Apply input
        config.input = args.value_of("INPUT").unwrap().into();

        // Apply other options and parameters
        config.quiet = args.is_present("quiet");

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input: Default::default(),
            output: Default::default(),
            profile: Default::default(),
            quiet: true,
        }
    }
}
