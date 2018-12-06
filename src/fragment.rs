use scene::MaterialIdx;
use vectors::Vec2d;
use vectors::Vec4d;

pub struct FragmentDifferentials {
    pub dpdx: Vec4d,
    pub dpdy: Vec4d,
}

pub struct Fragment {
    pub position: Vec4d,
    pub normal: Vec4d,
    pub uv: Vec2d,

    pub dpdu: Vec4d,
    pub dpdv: Vec4d,

    pub view: Vec4d, // Camera -> Point

    pub material: Option<MaterialIdx>,
}
