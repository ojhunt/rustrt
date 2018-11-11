use bounding_box::HasBoundingBox;
use collision::Collision;
use ray::Ray;
use std::fmt::Debug;

pub trait Intersectable: Debug + HasBoundingBox {
    fn intersect(&self, ray: Ray, min: f64, max: f64) -> Option<Collision>;
}

impl<T: Intersectable + ?Sized> Intersectable for Box<T> {
    fn intersect(&self, ray: Ray, min: f64, max: f64) -> Option<Collision> {
        return (**self).intersect(ray, min, max);
    }
}
