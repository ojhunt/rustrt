use std::fmt::Debug;
use ray::Ray;
use collision::Collision;
use bounding_box::HasBoundingBox;

pub trait Intersectable : Debug + HasBoundingBox {
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision>;
}

impl <T:Intersectable + ?Sized> Intersectable for Box<T> {
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision> {
        return (*self).intersect(ray, max);
    }
}
