use std::ops;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
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
