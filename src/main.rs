extern crate clap;
extern crate genmesh;
extern crate image;
extern crate itertools;
extern crate obj;

mod bounding_box;
mod bvh;
mod camera;
mod casefopen;
mod collision;
mod colour;
mod compound_object;
mod fragment;
mod intersectable;
mod material;
mod mesh;
mod objects;
mod ray;
mod scene;
mod shader;
mod triangle;
mod vectors;
mod wavefront_material;

use camera::*;
use clap::*;
use scene::*;
use std::str::FromStr;
use vectors::Vec4d;

struct SceneSettings {
    pub output_file: String,
    pub scene_file: String,
    pub camera_position: Vec4d,
    pub camera_target: Vec4d,
    pub camera_up: Vec4d,
}

impl SceneSettings {
    pub fn new() -> SceneSettings {
        return SceneSettings {
            output_file: String::new(),
            scene_file: String::new(),
            camera_position: Vec4d::point(0., 0.5, 0.),
            camera_target: Vec4d::point(0., 0., 10000000.),
            camera_up: Vec4d::vector(0.0, 1.0, 0.0),
        };
    }
}

#[derive(Debug)]
struct VecArg {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl VecArg {
    #[allow(dead_code)]
    pub fn as_vector(&self) -> Vec4d {
        Vec4d::vector(self.x, self.y, self.z)
    }
    pub fn as_point(&self) -> Vec4d {
        Vec4d::point(self.x, self.y, self.z)
    }
}

impl FromStr for VecArg {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self> {
        let coords: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .collect();

        let x = coords[0].parse::<f64>().unwrap();
        let y = coords[1].parse::<f64>().unwrap();
        let z = coords[2].parse::<f64>().unwrap();

        Ok(VecArg { x, y, z })
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
    match value_t!(matches, "position", VecArg) {
        Ok(value) => settings.camera_position = value.as_point(),
        _ => {}
    }
    match value_t!(matches, "target", VecArg) {
        Ok(value) => settings.camera_target = value.as_point(),
        _ => {}
    }
    return settings;
}

fn main() {
    let settings = load_settings();

    const SIZE: usize = 700;
    let scn = load_scene(&settings.scene_file);
    let camera = PerspectiveCamera::new(
        SIZE,
        SIZE,
        settings.camera_position,
        settings.camera_target,
        settings.camera_up,
        40.,
    );

    let start = std::time::Instant::now();
    let output = scn.render(&camera, SIZE);
    let end = std::time::Instant::now();
    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Time taken: {}", time);
    println!("Writing {}", settings.output_file);

    output.save(settings.output_file).unwrap();
    println!("Done!");
}
