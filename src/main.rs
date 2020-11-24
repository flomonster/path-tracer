#[macro_use]
extern crate clap;

mod config;
mod renderer;
mod scene;
mod utils;

pub use config::Config;

use clap::App;
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
    let yaml = load_yaml!("cli.yaml");
    let config = Config::load(&App::from_yaml(yaml).get_matches())?;

    let scene = Scene::load(&config)?;

    // Send scene to Renderer
    let renderer = Renderer::new(&config);
    let now = Instant::now();
    let rendered_image = renderer.render(&scene);
    println!("Render time: {}ms", now.elapsed().as_millis());

    // Save image
    rendered_image.save(config.output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn test_scene<P>(path: P)
    where
        P: AsRef<Path>,
    {
        let mut config = Config::default();
        config.input = PathBuf::from(path.as_ref());
        let scene = Scene::load(&config).unwrap();
        Renderer::new(&config).render(&scene);
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
