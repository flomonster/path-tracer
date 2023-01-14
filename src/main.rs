mod config;
mod renderer;
mod scene;

use clap::Parser;
use config::{Config, ConvertConfig, Profile, RenderConfig};
use renderer::debug_renderer::debug_render;
use renderer::Renderer;
use scene::internal::Scene;
use scene::{convert_gltf_to_isf, load_internal};
use std::error::Error;
use std::process::exit;

fn main() {
    match run() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    match config {
        Config::Render(render_config) => run_render(render_config),
        Config::Convert(convert_config) => run_convert(convert_config),
    }
}

fn run_render(config: RenderConfig) -> Result<(), Box<dyn Error>> {
    let profile = match &config.profile {
        Some(path) => Profile::load(path)?,
        None => Default::default(),
    };

    let scene = load_internal(&config.input)?;

    if config.debug_textures {
        debug_render(&scene, profile.resolution);
        return Ok(());
    }

    // Send scene to Renderer
    let renderer = Renderer::new(&config, profile);
    let rendered_image = renderer.render(&scene);

    // Save image
    rendered_image.save(config.output)?;
    Ok(())
}

fn run_convert(config: ConvertConfig) -> Result<(), Box<dyn Error>> {
    convert_gltf_to_isf(config.input, config.output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use image::EncodableLayout;
    use sha1::{digest::Update, Digest, Sha1};

    use crate::config::Resolution;

    use super::*;
    use std::path::Path;
    use std::str;

    fn test_scene<P>(path: P, expected_hash: &str)
    where
        P: AsRef<Path>,
    {
        let config = RenderConfig {
            input: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let scene = load_internal(&config.input).unwrap();
        let profile = Profile {
            resolution: Resolution {
                width: 800,
                height: 600,
            },
            bounces: 4,
            samples: 16,
            ..Default::default()
        };
        let image = Renderer::new(&config, profile).render(&scene);
        let hash = Sha1::new().chain(image.as_bytes()).finalize();
        assert_eq!(format!("{:02x}", &hash), expected_hash);
    }

    #[test]
    fn cube() {
        test_scene(
            "tests/scenes/cube/scene.isf",
            "56fddd23f8163fc3e962132796661b7e65649889",
        );
    }

    #[test]
    fn reflection() {
        test_scene(
            "tests/scenes/reflection/scene.isf",
            "87075c6dcc591d71a40f1c19b90b1cea60574662",
        );
    }

    #[test]
    fn head() {
        test_scene(
            "tests/scenes/head/scene.isf",
            "0f413dbd387f9bace28de6a3dcd29fbfc4ad2b0d",
        );
    }

    #[test]
    fn spheres() {
        test_scene(
            "tests/scenes/spheres/scene.isf",
            "a7354168996aab12d0edb93bd2ba3f67a505f701",
        );
    }

    #[test]
    fn alpha_transparency() {
        test_scene(
            "tests/scenes/alpha_transparency/scene.isf",
            "35d5b462a4f45fd445a7d6611287c4583e7beed9",
        );
    }
}
