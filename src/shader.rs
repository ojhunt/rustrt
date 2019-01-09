use collision::Collision;
use fragment::Fragment;
use ray::Ray;
use scene::Scene;
use vectors::Vec4d;

pub struct LightSample {
  pub position: Vec4d,
  pub direction: Option<Vec4d>,
  pub diffuse: Vec4d,
  pub specular: Vec4d,
  pub emission: Vec4d,
  pub weight: f64,
}

pub trait Light {
  fn get_area(&self) -> f64;
  fn get_samples(&self, count: usize, scene: &Scene) -> Vec<LightSample>;
}

pub trait Shadable {
  fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> Fragment;
}
