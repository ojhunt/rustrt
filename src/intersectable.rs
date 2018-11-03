use std::fmt::Debug;
use ray::Ray;
use collision::Collision;

pub trait Intersectable : Debug {
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision>;
}
