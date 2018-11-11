extern crate genmesh;
extern crate image;
extern crate itertools;

extern crate clap;

mod bounding_box;
mod bvh;
mod camera;
mod collision;
mod compound_object;
mod fragment;
mod intersectable;
mod mesh;
mod objects;
mod ray;
mod scene;
mod shader;
mod triangle;
mod vec4d;

use camera::Camera;
use clap::*;
use scene::*;
use vec4d::Vec4d;

extern crate obj;

struct SceneSettings {
    pub output_file: String,
    pub scene_file: String,
}

impl SceneSettings {
    pub fn new() -> SceneSettings {
        return SceneSettings {
            output_file: String::new(),
            scene_file: String::new(),
        };
    }
}

fn load_settings() -> SceneSettings {
    let commandline_yaml = load_yaml!("command_line.yml");
    let matches = App::from_yaml(commandline_yaml).get_matches();
    let output_file = matches.value_of("output").unwrap();
    let scene_file = matches.value_of("scene").unwrap();
    let mut settings = SceneSettings::new();
    settings.output_file = output_file.to_string();
    settings.scene_file = scene_file.to_string();
    return settings;
}

fn main() {
    let settings = load_settings();

    const SIZE: usize = 700;
    let scn = load_scene(&settings.scene_file);
    let camera = Camera::new(Vec4d::point(10., 1., 0.), Vec4d::point(0.0, 3.0, 0.0), 40.);

    let start = std::time::Instant::now();
    let output = scn.render(&camera, SIZE);
    let end = std::time::Instant::now();
    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Time taken: {}", time);

    output.save(settings.output_file).unwrap();
    println!("Done!");
}
