#[macro_use]
extern crate clap;

mod raytracer;
mod scene;
mod utils;

use cgmath::*;
use clap::App;
use raytracer::Raytracer;
use scene::Scene;
use std::process::exit;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut scene = match Scene::load(matches.value_of("INPUT").unwrap()) {
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
        Ok(scene) => scene,
    };

    // Add lights manually
    scene.lights.push(scene::Light::Directional(
        Vector3::new(-1., -1., 0.),
        Vector3::new(1., 1., 1.),
        1.,
    ));

    // Send scene to Raytracer
    let raytracer = Raytracer::new(300, 150);
    let rendered_image = raytracer.render(&scene);

    // Save image
    if let Err(e) = rendered_image.save(matches.value_of("OUTPUT").unwrap()) {
        eprintln!("{}", e);
        exit(2);
    }
}
