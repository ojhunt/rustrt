use collision::Collision;
use fragment::Fragment;
use ray::Ray;
use scene::Scene;

pub trait Shadable {
    fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> Fragment;
}
