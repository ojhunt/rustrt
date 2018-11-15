use collision::Collision;
use fragment::Fragment;
use ray::Ray;
use scene::Scene;

pub trait Shader {
    fn generate_secondary_rays(&self, fragment: Fragment) -> Vec<Ray>;
    fn shade(&self, samples: &[(f64, f64, f64)]) -> (f64, f64, f64);
}

pub trait Shadable {
    fn compute_fragment(&self, &Scene, Ray, Collision) -> Fragment;
}
