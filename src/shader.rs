use fragment::Fragment;
use ray::Ray;

pub trait Shader {
    fn generate_secondary_rays(&self, fragment: Fragment) -> Vec<Ray>;
    fn shade(&self, samples: &[(f64, f64, f64)]) -> (f64, f64, f64);
}
