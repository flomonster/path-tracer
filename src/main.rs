#[macro_use]
extern crate clap;
use clap::App;

mod scene;
use scene::Scene;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let scene = Scene::load(matches.value_of("INPUT").unwrap());
}