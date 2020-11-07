#[macro_use]
extern crate clap;

mod config;
mod raytracer;
mod scene;
mod utils;

pub use config::Config;

use cgmath::*;
use clap::App;
use raytracer::Raytracer;
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
    let config: Config = Config::load(&App::from_yaml(yaml).get_matches())?;

    let mut scene = Scene::load(&config)?;

    // Add lights manually

    scene.lights.push(scene::Light::new_directional(
        Vector3::new(-0., -1., -1.), // Direction
        Vector3::new(1., 1., 1.),    // Color
        1.,                          // Intensity
    ));

    /*
    scene.lights.push(scene::Light::new_point(
        Vector3::new(0.2, 3., 3.), // Position
        Vector3::new(1., 1., 1.),  // Color
        500.,                      // Intensity
    ));*/

    // Send scene to Raytracer
    let raytracer = Raytracer::new(&config);
    let rendered_image = raytracer.render(&scene);

    // Save image
    rendered_image.save(config.output)?;
    Ok(())
}
