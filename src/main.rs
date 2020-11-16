#[macro_use]
extern crate clap;

mod config;
mod raytracer;
mod scene;
mod utils;

pub use config::Config;

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
    let config = Config::load(&App::from_yaml(yaml).get_matches())?;

    let scene = Scene::load(&config)?;

    // Send scene to Raytracer
    let raytracer = Raytracer::new(&config);
    let rendered_image = raytracer.render(&scene);

    // Save image
    rendered_image.save(config.output)?;
    Ok(())
}
