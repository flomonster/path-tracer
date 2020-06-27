#[macro_use]
extern crate clap;

mod scene;
mod utils;

use clap::App;
use scene::Scene;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let scene = Scene::load(matches.value_of("INPUT").unwrap());
}
