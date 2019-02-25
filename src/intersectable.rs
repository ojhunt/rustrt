use crate::light::Light;
use crate::bounding_box::HasBoundingBox;
use crate::collision::Collision;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::shader::*;
use std::fmt::Debug;

#[derive(Copy, Clone, PartialEq)]
pub enum HitMode {
  AnyHit,
  Nearest,
}

pub trait Intersectable: Debug + HasBoundingBox + Sync + Send {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light>;
  fn intersect<'a>(&'a self, ray: &Ray, hit_mode: HitMode, min: f32, max: f32) -> Option<(Collision, &'a Shadable)>;
}

impl<T: Intersectable + ?Sized> Intersectable for Box<T> {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light> {
    return (**self).get_lights(s);
  }

  fn intersect<'a>(&'a self, ray: &Ray, hit_mode: HitMode, min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
    return (**self).intersect(ray, hit_mode, min, max);
  }
}
