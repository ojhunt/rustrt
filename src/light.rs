use crate::material::EmissionCoefficients;
use crate::vectors::Vector;
use crate::vectors::Point;
use crate::scene::Scene;

#[derive(Debug)]
pub struct LightSample {
  pub position: Point,
  pub direction: Option<Vector>,
  pub ambient: Vector,
  pub diffuse: Vector,
  pub specular: Vector,
  pub emission: EmissionCoefficients,
  pub weight: f32,
  pub power: f32,
}

pub trait Light {
  fn get_area(&self) -> f32;
  fn get_samples(&self, count: usize, scene: &Scene) -> Vec<LightSample>;
}
impl LightSample {
  pub fn output(&self) -> f32 {
    self.power * self.weight
  }
}
