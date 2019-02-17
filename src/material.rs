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
  pub ambient: f32,
  pub diffuse: f32,
  pub specular: f32,
}

impl EmissionCoefficients {
  pub fn max_value(&self) -> f32 {
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
  pub secondaries: Vec<(Ray, Colour, f32)>,
}

pub trait Material: Debug + Sync + Send {
  fn is_light(&self) -> bool;
  fn compute_surface_properties(&self, s: &Scene, ray: &Ray, f: &Fragment) -> MaterialCollisionInfo;
}

#[derive(Debug)]
pub struct DefaultMaterial {
  colour: Colour,
  reflection: Option<f32>,
}

impl DefaultMaterial {
  pub fn new(colour: Colour, reflection: Option<f32>) -> DefaultMaterial {
    DefaultMaterial { colour, reflection }
  }
}
impl Material for DefaultMaterial {
  fn is_light(&self) -> bool {
    false
  }

  fn compute_surface_properties(&self, _s: &Scene, ray: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    let reflected_direction = f.view.reflect(f.normal);
    let reflected_ray = Ray::new(
      f.position + (reflected_direction * 0.01),
      reflected_direction,
      Some(ray.ray_context.clone()),
    );
    MaterialCollisionInfo {
      ambient_colour: self.colour,
      diffuse_colour: self.colour,
      specular_colour: self.colour,
      emissive_colour: None,
      transparent_colour: None,
      position: f.position,
      normal: f.normal,
      secondaries: if let Some(_) = self.reflection {
        vec![(reflected_ray, Colour::RGB(0.95, 0.95, 0.95), 1.0)]
      } else {
        vec![]
      },
    }
  }
}
