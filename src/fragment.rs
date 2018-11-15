use vec4d::Vec4d;

pub struct Fragment {
    pub position: Vec4d,
    pub normal: Vec4d,
    pub tex_coord: [f64; 2],
}
