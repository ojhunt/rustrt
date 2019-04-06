#![feature(stdsimd, async_await, futures_api, await_macro, drain_filter, box_syntax)]
#![allow(unused)]

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
mod media;
mod mesh;
mod objects;
mod ray;
mod render_configuration;
mod scene_loader;
mod shader;
mod sphere;
mod texture;
mod triangle;
mod wavefront_material;

pub mod photon_map;
pub mod scene;
pub mod vectors;

pub use crate::render_configuration::RenderConfiguration;

pub mod wavefront {
  pub use crate::wavefront_material::load_scene;
}

pub use crate::camera::*;

pub mod integrators {
  pub use crate::direct_lighting::DirectLighting;
  pub use crate::direct_lighting::IndirectLightingSource;
  pub use crate::render_configuration::LightingIntegrator;
}

pub mod cameras {
  pub use crate::camera::Camera;
  pub use crate::camera::PerspectiveCamera;
}
