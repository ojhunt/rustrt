use vectors::Vec4d;

#[derive(Debug, Clone)]
pub struct RayContext {
  ior: Vec<f64>,
}

impl RayContext {
  pub fn new() -> RayContext {
    RayContext { ior: vec![] }
  }

  pub fn enter_material(&self, ior: f64) -> RayContext {
    let mut result = self.clone();

    return result;
  }

  pub fn current_ior_or(&self, or: f64) -> f64 {
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
  pub origin: Vec4d,
  pub direction: Vec4d,
  pub min: f64,
  pub max: f64,
  pub ray_context: RayContext,
}

fn clone_context(ctx: Option<RayContext>) -> RayContext {
  match ctx {
    None => RayContext::new(),
    Some(c) => c,
  }
}

impl Ray {
  pub fn new(origin: Vec4d, direction: Vec4d, ctx: Option<RayContext>) -> Ray {
    assert!(origin.w() == 1.0);
    assert!(direction.w() == 0.0);
    Ray {
      origin: origin,
      direction: direction,
      min: 0.0,
      max: std::f64::INFINITY,
      ray_context: clone_context(ctx),
    }
  }

  pub fn new_bound(origin: Vec4d, direction: Vec4d, min: f64, max: f64, ctx: Option<RayContext>) -> Ray {
    assert!(origin.w() == 1.0);
    assert!(direction.w() == 0.0);
    Ray {
      origin: origin,
      direction: direction,
      min: min,
      max: max,
      ray_context: clone_context(ctx),
    }
  }
}
