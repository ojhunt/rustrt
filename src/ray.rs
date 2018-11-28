use vectors::Vec4d;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec4d,
    pub direction: Vec4d,
    pub min: f64,
    pub max: f64,
}

impl Ray {
    pub fn new(origin: Vec4d, direction: Vec4d) -> Ray {
        assert!(origin.w == 1.0);
        assert!(direction.w == 0.0);
        Ray {
            origin: origin,
            direction: direction,
            min: 0.0,
            max: std::f64::INFINITY,
        }
    }
    pub fn new_bound(origin: Vec4d, direction: Vec4d, min: f64, max: f64) -> Ray {
        assert!(origin.w == 1.0);
        assert!(direction.w == 0.0);
        Ray {
            origin: origin,
            direction: direction,
            min: min,
            max: max,
        }
    }
}
