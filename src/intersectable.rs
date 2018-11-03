use ray::Ray;
use collision::Collision;

pub trait Intersectable {
    fn intersects(&self, ray: &Ray, max: f64) -> Option<Collision>;
}
