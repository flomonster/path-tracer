mod config;
mod renderer;
mod scene;
mod utils;

use clap::Parser;
use config::{Config, Profile};
use renderer::debug_renderer::debug_render;
use renderer::Renderer;
use scene::Scene;
use std::error::Error;
use std::process::exit;

fn main() {
    match run() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    let profile = match &config.profile {
        Some(path) => Profile::load(path)?,
        None => Default::default(),
    };

    let scene = Scene::load(&config.input)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_scene<P>(path: P)
    where
        P: AsRef<Path>,
    {
        let config = Config {
            input: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let scene = Scene::load(&config.input).unwrap();
        Renderer::new(&config, Default::default()).render(&scene);
    }

    #[test]
    fn cube() {
        test_scene("tests/scenes/cube.glb");
    }

    #[test]
    fn reflection() {
        test_scene("tests/scenes/reflection.glb");
    }

    #[test]
    fn head() {
        test_scene("tests/scenes/head.glb");
    }
}
