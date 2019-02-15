use crate::vectors::*;

#[derive(Debug, Clone)]
pub struct RayContext {
  ior: Vec<f32>,
}

impl RayContext {
  pub fn new() -> RayContext {
    RayContext { ior: vec![] }
  }

  pub fn enter_material(&self, _ior: f32) -> RayContext {
    self.clone()
  }

  pub fn current_ior_or(&self, _or: f32) -> f32 {
    *self.ior.last().unwrap_or(&1.0)
  }
  pub fn exit_material(&self) -> RayContext {
    let mut result = self.clone();
    result.ior.pop();
    return result;
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
