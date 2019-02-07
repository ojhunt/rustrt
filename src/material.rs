use crate::colour::Colour;
use crate::fragment::Fragment;
use crate::ray::Ray;
use crate::scene::Scene;
use std::fmt::Debug;
use crate::vectors::*;

#[derive(Debug, Clone, Copy)]
pub enum Transparency {
  Opaque,
  Constant(f64),
  // Halo(f64), // 1.0 - (N*v)(1.0-factor)
}

#[derive(Debug, Clone, Copy)]
pub struct EmissionCoefficients {
  pub ambient: f64,
  pub diffuse: f64,
  pub specular: f64,
}

impl EmissionCoefficients {
  pub fn new() -> EmissionCoefficients {
    return EmissionCoefficients {
      ambient: 0.0,
      diffuse: 0.0,
      specular: 0.0,
    };
  }
  pub fn max_value(&self) -> f64 {
    return self.ambient.max(self.diffuse).max(self.specular);
  }
}

#[derive(Clone)]
pub struct MaterialCollisionInfo {
  pub ambient_colour: Colour,
  pub diffuse_colour: Colour,
  pub specular_colour: Colour,
  pub emissive_colour: Option<EmissionCoefficients>,
  pub transparent_colour: Option<Colour>,
  pub position: Point,
  pub normal: Vector,
  pub secondaries: Vec<(Ray, Colour, f64)>,
}

pub trait Material: Debug + Sync + Send {
  fn is_light(&self) -> bool;
  fn compute_surface_properties(&self, s: &Scene, ray: &Ray, f: &Fragment) -> MaterialCollisionInfo;
}

#[derive(Debug)]
pub struct DefaultMaterial {
  colour: Colour,
}

impl DefaultMaterial {
  pub fn new(colour: Colour) -> DefaultMaterial {
    DefaultMaterial { colour }
  }
}
impl Material for DefaultMaterial {
  fn is_light(&self) -> bool {
    false
  }

  fn compute_surface_properties(&self, _s: &Scene, _ray: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    MaterialCollisionInfo {
      ambient_colour: self.colour,
      diffuse_colour: self.colour,
      specular_colour: self.colour,
      emissive_colour: None,
      transparent_colour: None,
      position: f.position,
      normal: f.normal,
      secondaries: vec![],
    }
  }
}
