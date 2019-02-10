#![feature(stdsimd, async_await, futures_api, await_macro, drain_filter)]

extern crate clap;
extern crate genmesh;
extern crate image;
extern crate itertools;
extern crate obj;
extern crate rand;
extern crate faster;
extern crate packed_simd;
extern crate rayon;
extern crate num_cpus;

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
mod light;
mod material;
mod mesh;
mod objects;
mod photon_map;
mod ray;
mod render_configuration;
mod scene;
mod shader;
mod texture;
mod triangle;
mod vectors;
mod wavefront_material;

use crate::render_configuration::LightingIntegrator;
use crate::camera::*;
use crate::photon_map::DiffuseSelector;
use crate::photon_map::PhotonMap;
use crate::photon_map::Timing;
use crate::scene::Scene;
use crate::scene::SceneSettings;
use crate::wavefront_material::load_scene;
use crate::render_configuration::RenderConfiguration;
use crate::vectors::*;
use clap::*;
use std::str::FromStr;
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
      settings.max_leaf_photons = value.max(4);
    }
    _ => {}
  }
  match value_t!(matches, "photon_samples", usize) {
    Ok(value) => {
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
    if settings.photon_samples == 0 {
      settings.use_direct_lighting = true;
    }
  }
  if matches.is_present("multisampling") {
    settings.use_multisampling = true;
  }
  return settings;
}

fn main() {
  let settings = load_settings();

  let mut scn = load_scene(&settings);
  {
    Scene::finalize(&mut scn, settings.max_leaf_photons);
  }

  let diffuse_map = Arc::new(DiffuseSelector::new(!settings.use_direct_lighting));
  let photonmap: Arc<Box<LightingIntegrator>> = Arc::new(Box::new(
    PhotonMap::new(
      &diffuse_map,
      &scn,
      &scn.get_light_samples(100000),
      settings.photon_count,
      settings.max_leaf_photons,
      settings.photon_samples,
    )
    .unwrap(),
  ));
  let camera = Box::new(PerspectiveCamera::new(
    settings.width,
    settings.height,
    settings.camera_position,
    settings.camera_target,
    settings.camera_up,
    40.,
    settings.samples_per_pixel,
    settings.use_multisampling,
  ));

  let configuration = Arc::new(RenderConfiguration::new(photonmap, scn, Arc::new(camera)));
  let output = {
    let _t = Timing::new("Total Rendering");
    let o = configuration.camera().render(&configuration);
    o
  };
  output.save(settings.output_file).unwrap();
}
