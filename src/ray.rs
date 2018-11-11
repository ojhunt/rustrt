use vec4d::Vec4d;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec4d,
    pub direction: Vec4d,
}

impl Ray {
    pub fn new(origin: Vec4d, direction: Vec4d) -> Ray {
        assert!(origin.w == 1.0);
        assert!(direction.w == 0.0);
        Ray {
            origin: origin,
            direction: direction,
        }
    }
}
