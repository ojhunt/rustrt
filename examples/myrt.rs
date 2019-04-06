#![feature(stdsimd, async_await, futures_api, await_macro, drain_filter, box_syntax)]
#![allow(unused)]

extern crate clap;
extern crate genmesh;
extern crate image;
extern crate obj;
extern crate rand;
extern crate packed_simd;
extern crate order_stat;
extern crate num_cpus;
extern crate xml;
extern crate sdl2;
extern crate raytrace_rs;

use std::time::Instant;
use raytrace_rs::RenderBuffer;

use raytrace_rs::cameras::*;
use raytrace_rs::integrators::*;
use raytrace_rs::photon_map::DiffuseSelector;
use raytrace_rs::photon_map::PhotonMap;
use raytrace_rs::photon_map::Timing;
use raytrace_rs::scene::Scene;
use raytrace_rs::scene::SceneSettings;
use raytrace_rs::wavefront::load_scene;
use raytrace_rs::RenderConfiguration;
use raytrace_rs::vectors::*;

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

/// Load the settings
struct RunSettings {
  scene_settings: SceneSettings,
  interactive: bool,
  output: Option<String>,
}
fn load_settings() -> RunSettings {
  let commandline_yaml = load_yaml!("command_line.yml");
  let matches = App::from_yaml(commandline_yaml).get_matches();
  let output_file = matches.value_of("output").map(|o| o.to_string());
  let scene_file = matches.value_of("scene").unwrap().to_string();

  let mut settings = SceneSettings::new();
  settings.scene_file = scene_file;
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

  return RunSettings {
    scene_settings: settings,
    interactive: matches.is_present("interactive") || output_file.is_none(),
    output: output_file,
  };
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
  let settings = load_settings();
  if settings.interactive {
    return run_interactive(&settings);
  }

  let settings = settings.scene_settings;
  let scn = Arc::new(load_scene(&settings));
  let lighting_integrator = lighting_integrator(&settings, &scn);
  let configuration = Arc::new(RenderConfiguration::new(lighting_integrator, scn));

  let camera = Box::new(PerspectiveCamera::new(
    settings.width as usize,
    settings.height as usize,
    settings.camera_position,
    settings.camera_direction,
    settings.camera_up,
    40.,
    settings.samples_per_pixel,
    settings.use_multisampling,
    settings.gamma,
  ));
  let output = camera.render(&configuration);

  return Ok(());
}

fn run_interactive(settings: &RunSettings) -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let video_subsystem = sdl_context.video()?;

  let window = video_subsystem
    .window(
      "rust-sdl2 demo: Window",
      settings.scene_settings.width as u32,
      settings.scene_settings.height as u32,
    )
    .resizable()
    .build()
    .map_err(|e| e.to_string())?;

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
  let mut gamma = settings.scene_settings.gamma;
  {
    let settings = settings.scene_settings.clone();
    thread::spawn(move || {
      let scn = Arc::new(load_scene(&settings));
      let lighting_integrator = lighting_integrator(&settings, &scn);
      let configuration = Arc::new(RenderConfiguration::new(lighting_integrator, scn));
      while let Ok(Some((camera, gamma))) = render_parameter_receiver.recv() {
        let start = Instant::now();
        let camera: Box<Camera> = camera;
        let output = {
          let _t = Timing::new("Total Rendering");
          let o = camera.render(&configuration);
          o
        };

        result_transmitter.send((
          output.width,
          output.height,
          output.to_pixel_array(gamma),
          Instant::now() - start,
        ));
      }
    });
  }

  let step_size = 0.3;
  let mut position = settings.scene_settings.camera_position;
  let mut render_count = 0;
  let mut render_time = 0;
  let (mut yaw, mut pitch) = vector_to_orientation(settings.scene_settings.camera_direction);
  'running: loop {
    if let Some(event) = event_pump.wait_event_timeout(1000 / 24) {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        }
        | Event::KeyDown {
          keycode: Some(Keycode::Q),
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
          keycode: Some(Keycode::LeftBracket),
          ..
        } => {
          should_render = true;
          gamma -= 0.1;
        }
        Event::KeyDown {
          keycode: Some(Keycode::RightBracket),
          ..
        } => {
          should_render = true;
          gamma += 0.1;
        }
        Event::KeyDown {
          keycode: Some(Keycode::W),
          ..
        } => {
          should_render = true;
          position = position + orientation_to_vector(yaw, pitch) * step_size;
        }
        Event::KeyDown {
          keycode: Some(Keycode::A),
          ..
        } => {
          should_render = true;
          let up = Vector::vector(0.0, 1.0, 0.0);
          let forward = orientation_to_vector(yaw, pitch);
          let left = up.cross(forward);
          position = position + left * step_size;
        }
        Event::KeyDown {
          keycode: Some(Keycode::D),
          ..
        } => {
          should_render = true;
          let up = Vector::vector(0.0, 1.0, 0.0);
          let forward = orientation_to_vector(yaw, pitch);
          let left = up.cross(forward);
          position = position - left * step_size;
        }
        Event::KeyDown {
          keycode: Some(Keycode::S),
          ..
        } => {
          should_render = true;
          position = position - orientation_to_vector(yaw, pitch) * step_size;
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
          settings.scene_settings.camera_up,
          40.,
          settings.scene_settings.samples_per_pixel,
          settings.scene_settings.use_multisampling,
          gamma,
        ));
        render_parameter_transmitter.send(Some((camera, settings.scene_settings.gamma)));
        rendering = true;
        should_render = false;
      }
    } else if let Ok((width, height, result_buffer, time)) =
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
      render_count += 1;
      render_time += time.as_millis();
    }
  }
  println!("Average render time: {}", render_time as f64 / render_count as f64);
  return Ok(());
}
