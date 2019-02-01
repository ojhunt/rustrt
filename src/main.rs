#![feature(stdsimd, async_await, futures_api, await_macro)]

extern crate clap;
extern crate genmesh;
extern crate image;
extern crate itertools;
extern crate obj;
extern crate rand;
extern crate faster;
extern crate packed_simd;
extern crate rayon;

mod bounding_box;
mod bvh;
mod camera;
mod casefopen;
mod collision;
mod colour;
mod compound_object;
mod dispatch_queue;
mod fragment;
mod heap;
mod intersectable;
mod kdtree;
mod material;
mod mesh;
mod objects;
mod photon_map;
mod ray;
mod scene;
mod shader;
mod texture;
mod triangle;
mod vectors;
mod wavefront_material;

use crate::scene::SceneSettings;
use crate::camera::*;
use clap::*;
use std::str::FromStr;
use crate::vectors::*;
use crate::wavefront_material::load_scene;
use std::sync::Arc;

#[derive(Debug)]
struct VecArg {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}
impl VecArg {
  #[allow(dead_code)]
  pub fn as_vector(&self) -> Vector {
    Vector::vector(self.x, self.y, self.z)
  }
  pub fn as_point(&self) -> Point {
    Vector::point(self.x, self.y, self.z)
  }
}

impl FromStr for VecArg {
  type Err = clap::Error;

  fn from_str(s: &str) -> Result<Self> {
    let coords: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split(',').collect();

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
  match value_t!(matches, "max_leaf_photons", usize) {
    Ok(value) => {
      println!("max-leaf-photons: {}", value);
      settings.max_leaf_photons = value.max(4);
    }
    _ => {}
  }
  match value_t!(matches, "photon_samples", usize) {
    Ok(value) => {
      println!("photon_samples: {}", value);
      settings.photon_samples = value;
    }
    _ => {}
  }
  match value_t!(matches, "height", usize) {
    Ok(value) => {
      settings.height = value;
    }
    _ => {}
  }
  match value_t!(matches, "width", usize) {
    Ok(value) => {
      settings.width = value;
    }
    _ => {}
  }
  match value_t!(matches, "photon_count", usize) {
    Ok(value) => {
      settings.photon_count = value;
    }
    _ => {}
  }

  match value_t!(matches, "samples_per_pixel", usize) {
    Ok(value) => {
      settings.samples_per_pixel = value.max(1);
    }
    _ => {}
  }
  if matches.is_present("use_direct_lighting") {
    settings.use_direct_lighting = true;
  } else {
    println!("photon samples2: {}", settings.photon_samples);
    if settings.photon_samples == 0 {
      settings.use_direct_lighting = true;
    }
  }
  return settings;
}

fn main() {
  let settings = load_settings();

  let mut scn = load_scene(&settings);
  {
    Arc::get_mut(&mut scn).unwrap().finalize(settings.max_leaf_photons);
  }
  let camera = PerspectiveCamera::new(
    settings.width,
    settings.height,
    settings.camera_position,
    settings.camera_target,
    settings.camera_up,
    40.,
    settings.samples_per_pixel,
  );

  let start = std::time::Instant::now();
  let output = camera.render(scn, settings.photon_samples);
  let end = std::time::Instant::now();
  let delta = end - start;
  let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
  println!("Time taken: {}", time);
  println!("Writing {}", settings.output_file);

  output.save(settings.output_file).unwrap();
  println!("Done!");
}
