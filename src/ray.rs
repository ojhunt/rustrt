use vec4d::Vec4d;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Ray {
    pub origin: Vec4d,
    pub direction: Vec4d
}

impl Ray {
    pub fn new(_origin: Vec4d, _direction: Vec4d) -> Ray {
        assert!(_origin.w == 1.0);
        assert!(_direction.w == 0.0);
        Ray {
            origin: _origin,
            direction: _direction
        }
    }
}
