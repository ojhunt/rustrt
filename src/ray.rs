use crate::media::Media;
use crate::vectors::*;

#[derive(Debug, Clone)]
pub struct RayContext {}

impl RayContext {
  pub fn new() -> RayContext {
    RayContext {}
  }

  pub fn enter_material(&self) -> RayContext {
    self.clone()
  }

  pub fn exit_material(&self) -> RayContext {
    return self.clone();
  }
}

#[derive(Debug, Clone)]
pub struct Ray {
  pub origin: Point,
  pub direction: Vector,
  pub min: f32,
  pub max: f32,
  pub ray_context: RayContext,
}

fn clone_context(ctx: Option<RayContext>) -> RayContext {
  match ctx {
    None => RayContext::new(),
    Some(c) => c,
  }
}

impl Ray {
  pub fn new(origin: Point, direction: Vector, ctx: Option<RayContext>) -> Ray {
    Ray {
      origin: origin,
      direction: direction,
      min: 0.0,
      max: std::f32::INFINITY,
      ray_context: clone_context(ctx),
    }
  }

  pub fn new_bound(origin: Point, direction: Vector, min: f32, max: f32, ctx: Option<RayContext>) -> Ray {
    Ray {
      origin: origin,
      direction: direction,
      min: min,
      max: max,
      ray_context: clone_context(ctx),
    }
  }
}
