use scene::MaterialIdx;
use vectors::Vec2d;
use vectors::Vec4d;

pub struct Fragment {
    pub position: Vec4d,
    pub normal: Vec4d,
    pub uv: Vec2d,
    pub du: Vec4d,
    pub dv: Vec4d,
    pub material: Option<MaterialIdx>,
}
