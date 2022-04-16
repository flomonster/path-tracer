mod profile;
mod resolution;

use clap::Parser;
use derivative::Derivative;
pub use profile::Profile;
pub use resolution::Resolution;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Derivative)]
#[derivative(Default)]
#[clap(author, version)]
/// Path-trace awesome things
pub struct Config {
    /// Input file name GLTF format
    pub input: PathBuf,
    /// Output image name
    #[clap(long, short, env, default_value = "render.png")]
    pub output: PathBuf,
    /// No progress bar printed
    #[clap(long, short)]
    #[derivative(Default(value = "true"))]
    pub quiet: bool,
    /// Display a viewer (might slow down the rendering)
    #[clap(long, short)]
    pub viewer: bool,
    /// Generate debug textures
    #[clap(long)]
    pub debug_textures: bool,
    /// A path to the yaml file containing all the rendering profile information
    #[clap(long, short, env)]
    pub profile: Option<PathBuf>,
}
