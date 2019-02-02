use crate::collision::Collision;
use crate::fragment::Fragment;
use crate::ray::Ray;
use crate::scene::Scene;

pub trait Shadable {
  fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> Fragment;
}
