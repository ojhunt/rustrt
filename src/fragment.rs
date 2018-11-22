use scene::MaterialIdx;
use vec4d::Vec4d;

pub struct Fragment {
    pub position: Vec4d,
    pub normal: Vec4d,
    pub uv: (f64, f64),
    pub material: Option<MaterialIdx>,
}
