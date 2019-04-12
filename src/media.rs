use crate::fragment::Fragment;
use crate::colour::Colour;
use crate::bounding_box::BoundingBox;
use crate::light::Light;
use crate::scene::Scene;
use crate::intersectable::HitMode;
use crate::bounding_box::HasBoundingBox;
use std::fmt::Debug;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::collision::Collision;
use crate::shader::Shadable;

pub struct MediaFragment {
  diffuse_colour: Colour,
  density: f32,
}

pub trait Media: Debug + Send + Sync {
  fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> MediaFragment;
}

#[derive(Debug)]
struct HomogenousMedia {
  density: f32,
  colour: Colour,
  bounds: BoundingBox,
}
