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

fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = Config::parse();
    match config {
        Config::Render(render_config) => run_render(render_config),
        Config::Convert(convert_config) => run_convert(convert_config),
    }
}

fn run_render(config: RenderConfig) -> Result<(), Box<dyn Error + Send + Sync>> {
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

fn run_convert(config: ConvertConfig) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    fn test_scene_with_profile<P>(path: P, expected_hash: &str, profile: Profile)
    where
        P: AsRef<Path>,
    {
        let config = RenderConfig {
            input: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let scene = load_internal(&config.input).unwrap();
        let image = Renderer::new(&config, profile).render(&scene);
        let hash = Sha1::new().chain(image.as_bytes()).finalize();
        assert_eq!(format!("{:02x}", &hash), expected_hash);
    }

    fn test_scene<P>(path: P, expected_hash: &str)
    where
        P: AsRef<Path>,
    {
        let profile = Profile {
            resolution: Resolution {
                width: 800,
                height: 600,
            },
            bounces: 4,
            samples: 16,
            ..Default::default()
        };
        test_scene_with_profile(path, expected_hash, profile);
    }

    #[test]
    fn cube() {
        test_scene(
            "tests/scenes/cube/scene.isf",
            "60558456ace7e8063ebfab219ee35a2c7de862f5",
        );
    }

    #[test]
    fn reflection() {
        test_scene(
            "tests/scenes/reflection/scene.isf",
            "6ccc3b9f20442f15f25c41cf8d342ede5185e3db",
        );
    }

    #[test]
    fn head() {
        test_scene(
            "tests/scenes/head/scene.isf",
            "2c90976144ba14fe9f06ec3c812ff30f0a0c9146",
        );
    }

    #[test]
    fn spheres() {
        test_scene(
            "tests/scenes/spheres/scene.isf",
            "fe2687e274ac978a4815f202612eca71ee8dd8c9",
        );
    }

    #[test]
    fn alpha_transparency() {
        test_scene(
            "tests/scenes/alpha_transparency/scene.isf",
            "fdf9ccbe9dc3f3102e3c05b96d2984000e73b62f",
        );
    }

    #[test]
    fn white_furnace_indirect() {
        test_scene(
            "tests/scenes/white_furnace_indirect/scene.isf",
            "80dd0598ced75660b80170e69cad1a74fba26a15",
        );
    }

    #[test]
    fn white_furnace_direct() {
        let profile = Profile {
            resolution: Resolution {
                width: 800,
                height: 600,
            },
            bounces: 0, // Direct lighting only
            samples: 16,
            ..Default::default()
        };

        test_scene_with_profile(
            "tests/scenes/white_furnace_direct/scene.isf",
            "6838e727798bd33f2f796be3edaa893445087159",
            profile,
        );
    }
}
