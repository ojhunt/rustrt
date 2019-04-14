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

pub struct MediaIntersection {
  diffuse_colour: Colour,
  density: f32,
}

pub trait Media: Debug + Send + Sync + Shadable {
  fn compute_media_fragment(&self, s: &Scene, r: &Ray) -> Option<(f32, MediaIntersection)>;
}

#[derive(Debug)]
struct HomogenousMedia {
  density: f32,
  colour: Colour,
  bounds: BoundingBox,
}
