use scene::MaterialIdx;
use vectors::Vec4d;

pub struct Fragment {
    pub position: Vec4d,
    pub normal: Vec4d,
    pub uv: (f64, f64),
    pub du: Vec4d,
    pub dv: Vec4d,
    pub material: Option<MaterialIdx>,
}
