use std::ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec4d {
    pub fn new() -> Vec4d {
        Vec4d {
            x: 0.,
            y: 0.,
            z: 0.,
            w: 0.,
        }
    }
    pub fn vector(x: f64, y: f64, z: f64) -> Vec4d {
        Vec4d {
            x: x,
            y: y,
            z: z,
            w: 0.0,
        }
    }
    pub fn point(x: f64, y: f64, z: f64) -> Vec4d {
        Vec4d {
            x: x,
            y: y,
            z: z,
            w: 1.0,
        }
    }
    pub fn cross(&self, rhs: Vec4d) -> Vec4d {
        assert!(self.w == 0. && rhs.w == 0.);
        Vec4d {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: 0.,
        }
    }
    pub fn square_length(&self) -> f64 {
        return self.dot(*self);
    }
    pub fn length(&self) -> f64 {
        return self.square_length().sqrt();
    }
    pub fn dot(&self, rhs: Vec4d) -> f64 {
        assert!(self.w == 0. && rhs.w == 0.);
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
    pub fn normalize(&self) -> Vec4d {
        let scale = 1.0 / self.dot(*self).sqrt();
        return *self * scale;
    }

    pub fn scale(self, scale: f64) -> Vec4d {
        Vec4d {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
            w: self.w * scale,
        }
    }

    pub fn add_elements(self, _rhs: Vec4d) -> Vec4d {
        Vec4d {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
            w: self.w + _rhs.w,
        }
    }
    pub fn min(self, rhs: Vec4d) -> Vec4d {
        assert!(self.w == rhs.w);
        Vec4d {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
            z: self.z.min(rhs.z),
            w: self.w,
        }
    }
    pub fn max(self, rhs: Vec4d) -> Vec4d {
        assert!(self.w == rhs.w);
        Vec4d {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
            z: self.z.max(rhs.z),
            w: self.w,
        }
    }
}

impl ops::Neg for Vec4d {
    type Output = Vec4d;
    fn neg(self) -> Vec4d {
        return self.scale(-1.0);
    }
}

impl ops::Mul<f64> for Vec4d {
    type Output = Vec4d;
    fn mul(self, rhs: f64) -> Vec4d {
        return self.scale(rhs);
    }
}

impl ops::Mul<Vec4d> for f64 {
    type Output = Vec4d;
    fn mul(self, rhs: Vec4d) -> Vec4d {
        return rhs.scale(self);
    }
}

impl ops::Add<Vec4d> for Vec4d {
    type Output = Vec4d;

    fn add(self, _rhs: Vec4d) -> Vec4d {
        assert!(self.w == 0. || _rhs.w == 0.);
        return self.add_elements(_rhs);
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

impl ops::Index<usize> for Vec4d {
    type Output = f64;

    fn index(&self, index: usize) -> &f64 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("invalid vector index"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2d(pub f64, pub f64);

impl Vec2d {
    pub fn scale(&self, s: f64) -> Vec2d {
        return Vec2d(self.0 * s, self.1 * s);
    }
    pub fn add_elements(&self, rhs: Vec2d) -> Vec2d {
        return Vec2d(self.0 + rhs.0, self.1 + rhs.1);
    }
}
impl ops::Mul<f64> for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs: f64) -> Vec2d {
        return self.scale(rhs);
    }
}

impl ops::Add<Vec2d> for Vec2d {
    type Output = Vec2d;

    fn add(self, _rhs: Vec2d) -> Vec2d {
        return self.add_elements(_rhs);
    }
}

impl ops::Sub<Vec2d> for Vec2d {
    type Output = Vec2d;

    fn sub(self, rhs: Vec2d) -> Vec2d {
        Vec2d(self.0 - rhs.0, self.1 - rhs.1)
    }
}

#[test]
fn test_dot() {
    assert_eq!(Vec4d::vector(0., 1., 0.).dot(Vec4d::vector(1., 0., 0.)), 0.);
}
#[test]
fn test_cross() {
    assert_eq!(
        Vec4d::vector(2., 1., -1.).cross(Vec4d::vector(-3., 4., 1.)),
        Vec4d::vector(5., 1., 11.)
    );
    assert_eq!(
        Vec4d::vector(-3., 4., 1.).cross(Vec4d::vector(2., 1., -1.)),
        Vec4d::vector(-5., -1., -11.)
    );
}

#[test]
fn test_normalize() {
    assert_eq!(
        Vec4d::vector(2., 0., 0.).normalize(),
        Vec4d::vector(1., 0., 0.)
    );
}
