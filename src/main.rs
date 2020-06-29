#[macro_use]
extern crate clap;

mod scene;

use clap::App;
use scene::Scene;
use std::process::exit;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let scene = Scene::load(matches.value_of("INPUT").unwrap());
    if let Err(e) = scene {
        eprintln!("{}", e);
        exit(2);
    }
    // Send scene to Raytracer
}
