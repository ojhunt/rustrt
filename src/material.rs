use crate::scene::MediaIdx;
use std::sync::Arc;
use crate::media::Media;
use crate::ray::RayContext;
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
  pub reflectivity: Option<(f32, Colour)>,
  pub index_of_refraction: Option<(f32, f32)>,
  pub position: Point,
  pub normal: Vector,
  pub media_transition: Option<MediaTransition>,
}

#[derive(Clone, Copy, Debug)]
pub struct MediaTransition {
  pub internal: Option<MediaIdx>,
  pub external: Option<MediaIdx>,
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

  fn compute_surface_properties(&self, _s: &Scene, _: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    MaterialCollisionInfo {
      ambient_colour: self.colour,
      diffuse_colour: self.colour,
      specular_colour: self.colour,
      emissive_colour: None,
      transparent_colour: None,
      position: f.position,
      normal: f.normal,
      index_of_refraction: None,
      reflectivity: self.reflection.map(|p| (p, self.colour)),
      media_transition: None,
    }
  }
}

#[derive(Debug)]
pub struct TransparentMaterial {
  ior: f32,
  colour: Colour,
}

impl Material for TransparentMaterial {
  fn is_light(&self) -> bool {
    false
  }

  fn compute_surface_properties(&self, _s: &Scene, _: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    MaterialCollisionInfo {
      ambient_colour: self.colour,
      diffuse_colour: self.colour,
      specular_colour: self.colour,
      emissive_colour: None,
      transparent_colour: Some(self.colour),
      position: f.position,
      normal: f.normal,
      index_of_refraction: Some((self.ior, 1.0)),
      reflectivity: None,
      media_transition: None,
    }
  }
}
impl TransparentMaterial {
  pub fn new(ior: f32) -> TransparentMaterial {
    TransparentMaterial {
      ior,
      colour: Colour::RGB(1.0, 1.0, 1.0),
    }
  }
}

pub fn compute_secondaries(ray: &Ray, fragment: &Fragment, surface: &MaterialCollisionInfo) -> Vec<(Ray, Colour, f32)> {
  if surface.transparent_colour.is_none() {
    if let Some((reflection_weight, reflection_colour)) = surface.reflectivity {
      let reflected_direction = fragment.view.reflect(surface.normal);
      let reflected_ray = Ray::new(
        surface.position + (reflected_direction * 0.01),
        reflected_direction,
        Some(ray.ray_context.clone()),
      );
      return vec![(reflected_ray, reflection_colour, reflection_weight)];
    } else {
      return vec![];
    }
  }

  let normal = surface.normal;
  let reflected_ray = fragment.view.reflect(normal);
  let transparent_colour = if let Some(transparent_colour) = surface.transparent_colour {
    Colour::RGB(1.0, 1.0, 1.0) - transparent_colour
  } else {
    Colour::RGB(1.0, 1.0, 1.0)
  };
  let reflected_colour = if let Some((_, reflected_colour)) = surface.reflectivity {
    reflected_colour
  } else {
    surface.diffuse_colour
  };

  let mut result = vec![];
  let mut refraction_weight = 1.0;
  let mut exiting_object = false;
  let (refracted_vector, new_context): (Vector, RayContext) = match surface.index_of_refraction {
    None => (fragment.view, ray.ray_context.clone()),
    Some((internal_ior, external_ior)) => {
      let view = fragment.view * -1.0;
      let in_object = fragment.view.dot(fragment.true_normal) < 0.0;
      let (ni, nt, new_context) = if in_object {
        exiting_object = true;
        let new_context = ray.ray_context.exit_material();
        (internal_ior, external_ior, new_context)
      } else {
        let new_context = ray
          .ray_context
          .enter_material(surface.media_transition.map(|m| m.internal).unwrap_or(None));
        (external_ior, internal_ior, new_context)
      };
      let nr = ni as f32 / nt as f32;

      let n_dot_v = normal.dot(view);

      let inner = 1.0 - nr * nr * (1.0 - n_dot_v * n_dot_v);
      if inner < 0.0 {
        (reflected_ray, ray.ray_context.clone())
      } else {
        // Schlick approximation of fresnel term
        let r0 = {
          let r0root = (nt - ni) / (nt + ni);
          r0root * r0root
        };
        let fresnel_weight = {
          let one_minus_cos_theta = 1.0 - n_dot_v;
          let squared = one_minus_cos_theta * one_minus_cos_theta;
          let quintupled = squared * squared * one_minus_cos_theta;
          r0 + (1.0 - r0) * quintupled
        };
        if fresnel_weight > 0.02 {
          result.push((
            Ray::new(
              fragment.position + reflected_ray * 0.01,
              reflected_ray,
              Some(ray.ray_context.clone()),
            ),
            surface.specular_colour,
            fresnel_weight,
          ));
          refraction_weight -= fresnel_weight;
        }
        (
          ((nr * n_dot_v - inner.sqrt()) * normal - nr * view).normalize(),
          new_context,
        )
      }
    }
  };
  result.push((
    Ray::new(
      fragment.position + refracted_vector * 0.01,
      refracted_vector,
      Some(new_context),
    ),
    transparent_colour,
    refraction_weight,
  ));

  return result;
}

#[derive(Debug)]
struct Fog {
  media: MediaIdx,
}

impl Material for Fog {
  fn is_light(&self) -> bool {
    false
  }
  fn compute_surface_properties(&self, s: &Scene, ray: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    let transition = MediaTransition {
      internal: Some(self.media),
      external: None,
    };
    return MaterialCollisionInfo {
      ambient_colour: Colour::RGB(1.0, 1.0, 1.0),
      diffuse_colour: Colour::RGB(1.0, 1.0, 1.0),
      specular_colour: Colour::RGB(1.0, 1.0, 1.0),
      emissive_colour: None,
      transparent_colour: Some(Colour::RGB(1.0, 1.0, 1.0)),
      position: f.position,
      normal: f.normal,
      index_of_refraction: Some((1.0, 1.0)),
      reflectivity: None,
      media_transition: Some(transition),
    };
  }
}
