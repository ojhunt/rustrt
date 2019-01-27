use collision::Collision;
use fragment::Fragment;
use ray::Ray;
use scene::Scene;
use vectors::{Point, Vector};

#[derive(Debug)]
pub struct LightSample {
  pub position: Point,
  pub direction: Option<Vector>,
  pub diffuse: Vector,
  pub specular: Vector,
  pub emission: Vector,
  pub weight: f64,
}

pub trait Light {
  fn get_area(&self) -> f64;
  fn get_samples(&self, count: usize, scene: &Scene) -> Vec<LightSample>;
}

pub trait Shadable {
  fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> Fragment;
}
