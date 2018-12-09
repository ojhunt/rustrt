use bounding_box::HasBoundingBox;
use collision::Collision;
use ray::Ray;
use scene::Scene;
use shader::*;
use std::fmt::Debug;

pub trait Intersectable: Debug + HasBoundingBox {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light>;
  fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)>;
}

impl<T: Intersectable + ?Sized> Intersectable for Box<T> {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light> {
    return (**self).get_lights(s);
  }
  fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)> {
    return (**self).intersect(ray, min, max);
  }
}
