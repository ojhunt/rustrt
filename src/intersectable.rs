use bounding_box::HasBoundingBox;
use collision::Collision;
use ray::Ray;
use shader::Shadable;
use std::fmt::Debug;

pub trait Intersectable: Debug + HasBoundingBox {
    fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)>;
}

impl<T: Intersectable + ?Sized> Intersectable for Box<T> {
    fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)> {
        return (**self).intersect(ray, min, max);
    }
}
