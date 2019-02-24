#![feature(stdsimd, async_await, futures_api, await_macro, drain_filter)]
#![allow(unused)]
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
extern crate order_stat;
extern crate xml;
extern crate sdl2;

mod bounding_box;
mod bvh;
mod camera;
mod casefopen;
mod collision;
mod colour;
mod compound_object;
mod direct_lighting;
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
mod scene_loader;
mod shader;
mod sphere;
mod texture;
mod triangle;
mod vectors;
mod wavefront_material;

use crate::camera::RenderBuffer;
use crate::render_configuration::LightingIntegrator;
use crate::camera::*;
use crate::direct_lighting::DirectLighting;
use crate::photon_map::DiffuseSelector;
use crate::photon_map::PhotonMap;
use crate::photon_map::Timing;
use crate::direct_lighting::IndirectLightingSource;
use crate::scene::Scene;
use crate::scene::SceneSettings;
use crate::wavefront_material::load_scene;
use crate::render_configuration::RenderConfiguration;
use crate::vectors::*;
use std::result::Result;
use clap::*;
use std::str::FromStr;
use std::sync::Arc;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::thread;
use std::sync::mpsc;

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

  fn from_str(s: &str) -> clap::Result<Self> {
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
  match value_t!(matches, "target", VecArg) {
    Ok(value) => settings.camera_direction = (value.as_point() - settings.camera_position).normalize(),
    _ => {}
  }
  match value_t!(matches, "direction", VecArg) {
    Ok(value) => settings.camera_direction = value.as_vector().normalize(),
    _ => {}
  }
  match value_t!(matches, "gamma", f32) {
    Ok(value) => settings.gamma = value,
    _ => {}
  }
  return settings;
}

fn lighting_integrator(settings: &SceneSettings, scene: &Arc<Scene>) -> Arc<LightingIntegrator> {
  let lights = scene.get_light_samples(10000);
  let photon_map = if settings.photon_count != 0 && settings.photon_samples != 0 {
    let diffuse_map = Arc::new(DiffuseSelector::new(!settings.use_direct_lighting));
    PhotonMap::new(
      &diffuse_map,
      scene,
      &lights,
      settings.photon_count,
      settings.max_leaf_photons,
      settings.photon_samples,
    )
  } else {
    None
  };

  if !settings.use_direct_lighting && photon_map.is_some() {
    return Arc::new(photon_map.unwrap());
  }
  let indirect_source: Option<Arc<IndirectLightingSource>> = photon_map.map(|p| {
    let p: Arc<IndirectLightingSource> = Arc::new(p);
    return p;
  });
  return Arc::new(DirectLighting::new(scene, lights, indirect_source));
}

fn vector_to_orientation(vector: Vector) -> (f32, f32) {
  let yaw = vector.x().atan2(vector.z());
  let pitch = (-vector.y()).asin();
  return (yaw, pitch);
}

fn orientation_to_vector(yaw: f32, pitch: f32) -> Vector {
  return Vector::vector(
    (yaw.sin() * pitch.cos()).into(),
    pitch.sin().into(),
    (yaw.cos() * pitch.cos()).into(),
  );
}

fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  let window = video_subsystem
    .window("rust-sdl2 demo: Window", 800, 600)
    .resizable()
    .build()
    .map_err(|e| e.to_string())?;

  let settings = load_settings();

  let mut canvas = window
    .into_canvas()
    .present_vsync()
    .build()
    .map_err(|e| e.to_string())?;

  let mut tick = 0;

  let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

  let (result_transmitter, result_receiver) = mpsc::channel();
  let (render_parameter_transmitter, render_parameter_receiver) = mpsc::channel();
  let mut rendering = false;
  let mut should_render = true;
  {
    let settings = settings.clone();
    thread::spawn(move || {
      let scn = Arc::new(load_scene(&settings));
      let lighting_integrator = lighting_integrator(&settings, &scn);
      let configuration = Arc::new(RenderConfiguration::new(lighting_integrator, scn));
      while let Ok(Some((camera, gamma))) = render_parameter_receiver.recv() {
        let camera: Box<Camera> = camera;
        let output = {
          let _t = Timing::new("Total Rendering");
          let o = camera.render(&configuration);
          o
        };

        result_transmitter.send((output.width, output.height, output.to_pixel_array(gamma)));
      }
    });
  }
  let mut position = settings.camera_position;
  let (mut yaw, mut pitch) = vector_to_orientation(settings.camera_direction);
  'running: loop {
    if let Some(event) = event_pump.wait_event_timeout(1000 / 24) {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        Event::KeyDown {
          keycode: Some(Keycode::Left),
          ..
        } => {
          yaw += 0.1;
          should_render = true;
        }
        Event::KeyDown {
          keycode: Some(Keycode::Right),
          ..
        } => {
          should_render = true;
          yaw -= 0.1;
        }
        Event::KeyDown {
          keycode: Some(Keycode::Up),
          ..
        } => {
          pitch += 0.1;
          should_render = true;
        }
        Event::KeyDown {
          keycode: Some(Keycode::Down),
          ..
        } => {
          should_render = true;
          pitch -= 0.1;
        }
        Event::KeyDown {
          keycode: Some(Keycode::W),
          ..
        } => {
          should_render = true;
          position = position + orientation_to_vector(yaw, pitch) * 10.0;
        }
        Event::KeyDown {
          keycode: Some(Keycode::S),
          ..
        } => {
          should_render = true;
          position = position - orientation_to_vector(yaw, pitch) * 10.0;
        }
        Event::Window {
          win_event: WindowEvent::Resized(..),
          ..
        } => {
          should_render = true;
        }
        _ => {}
      }
    }
    {
      // Update the window title.
      let window = canvas.window_mut();
      let position = window.position();
      let size = window.size();
      let title = format!(
        "Window - pos({}x{}), size({}x{}): {}",
        position.0, position.1, size.0, size.1, tick
      );
      window.set_title(&title).map_err(|e| e.to_string())?;

      tick += 1;
    }

    if !rendering {
      if should_render {
        let window = canvas.window();
        let (width, height) = window.size();

        let camera = Box::new(PerspectiveCamera::new(
          width as usize,
          height as usize,
          position,
          orientation_to_vector(yaw, pitch),
          settings.camera_up,
          40.,
          settings.samples_per_pixel,
          settings.use_multisampling,
          settings.gamma,
        ));
        render_parameter_transmitter.send(Some((camera, settings.gamma)));
        rendering = true;
        should_render = false;
      }
    } else if let Ok((width, height, result_buffer)) =
      result_receiver.recv_timeout(std::time::Duration::from_millis(50))
    {
      rendering = false;
      let texture_creator = canvas.texture_creator();
      let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, width as u32, height as u32)
        .map_err(|e| e.to_string())?;
      texture.update(Rect::new(0, 0, width as u32, height as u32), &result_buffer, width * 3);
      canvas.clear();

      canvas.copy(&texture, None, None)?;
      canvas.present();
    }
  }
  return Ok(());
}
