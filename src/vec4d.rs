use std::ops;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Vec4d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64
}

impl Vec4d {
    pub fn vector(x: f64, y: f64, z: f64) -> Vec4d {
        Vec4d {x: x, y: y, z:z, w:0.0}
    }
    pub fn point(x: f64, y: f64, z: f64) -> Vec4d {
        Vec4d {x: x, y: y, z:z, w:1.0}
    }
    pub fn cross(&self, rhs: Vec4d) -> Vec4d {
        assert!(self.w == 0. && rhs.w == 0.);
        Vec4d {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: 0.
        }
    }
    pub fn dot(&self, rhs: Vec4d) -> f64 {
        assert!(self.w == 0. && rhs.w == 0.);
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
    pub fn normalize(&self) -> Vec4d {
        let scale = 1.0 / self.dot(*self).sqrt();
        return *self * scale;
    }
}

impl ops::Mul<f64> for Vec4d {
    type Output = Vec4d;
    fn mul(self, rhs: f64) -> Vec4d {
        Vec4d{
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w
        }
    }
}
impl ops::Add<Vec4d> for Vec4d {
    type Output = Vec4d;

    fn add(self, _rhs: Vec4d) -> Vec4d {
        assert!(self.w == 0. || _rhs.w == 0.);
        Vec4d {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
            w: self.w + _rhs.w,
        }
    }
}

impl ops::Sub<Vec4d> for Vec4d {
    type Output = Vec4d;

    fn sub(self, _rhs: Vec4d) -> Vec4d {
        assert!(self.w == 1. || _rhs.w == 0.);
        Vec4d {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
            z: self.z - _rhs.z,
            w: self.w - _rhs.w,
        }
    }
}

#[test]
fn test_dot() {
    assert_eq!(Vec4d::vector(0., 1., 0.).dot(Vec4d::vector(1.,0.,0.)), 0.);
}
#[test]
fn test_cross() {
    assert_eq!(Vec4d::vector(2., 1., -1.).cross(Vec4d::vector(-3.,4.,1.)), Vec4d::vector(5., 1., 11.));
    assert_eq!(Vec4d::vector(-3.,4.,1.).cross(Vec4d::vector(2., 1., -1.)), Vec4d::vector(-5., -1., -11.));
}

#[test]
fn test_normalize() {
    assert_eq!(Vec4d::vector(2., 0., 0.).normalize(), Vec4d::vector(1., 0., 0.));
}