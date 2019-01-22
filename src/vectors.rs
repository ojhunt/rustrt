use std::ops;
use packed_simd::shuffle;
use faster::arch::x86::vecs::f32x4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4d {
  pub data: f32x4,
}

impl Vec4d {
  pub fn x(&self) -> f32 {
    self.data.extract(0)
  }
  pub fn y(&self) -> f32 {
    self.data.extract(1)
  }
  pub fn z(&self) -> f32 {
    self.data.extract(2)
  }
  pub fn w(&self) -> f32 {
    self.data.extract(3)
  }
  pub fn new() -> Vec4d {
    Vec4d {
      data: f32x4::splat(0.0),
    }
  }

  pub fn vector(x: f64, y: f64, z: f64) -> Vec4d {
    Vec4d {
      data: f32x4::new(x as f32, y as f32, z as f32, 0.0),
    }
  }

  pub fn point(x: f64, y: f64, z: f64) -> Vec4d {
    Vec4d {
      data: f32x4::new(x as f32, y as f32, z as f32, 1.0),
    }
  }

  pub fn reflect(&self, normal: Vec4d) -> Vec4d {
    assert!(self.dot(normal) <= 0.0);
    (-2.0 * self.dot(normal) * normal + *self).normalize()
  }

  pub fn cross(&self, rhs: Vec4d) -> Vec4d {
    let rhs201: f32x4 = shuffle!(rhs.data, [2, 0, 1, 0]);
    let rhs120: f32x4 = shuffle!(rhs.data, [1, 2, 0, 0]);
    let lhs120: f32x4 = shuffle!(self.data, [1, 2, 0, 0]);
    let lhs201: f32x4 = shuffle!(self.data, [2, 0, 1, 0]);

    Vec4d {
      data: lhs120 * rhs201 - lhs201 * rhs120,
    }
  }
  pub fn square_length(&self) -> f64 {
    return self.dot(*self);
  }
  pub fn length(&self) -> f64 {
    return self.square_length().sqrt();
  }
  pub fn dot(&self, rhs: Vec4d) -> f64 {
    let scaled = self.data * rhs.data;
    return scaled.sum() as f64;
  }
  pub fn normalize(&self) -> Vec4d {
    let scale = 1.0 / self.dot(*self).sqrt();
    return *self * scale;
  }

  pub fn scale(self, scale: f64) -> Vec4d {
    Vec4d {
      data: self.data * scale as f32,
    }
  }

  pub fn add_elements(self, _rhs: Vec4d) -> Vec4d {
    Vec4d {
      data: self.data + _rhs.data,
    }
  }
  pub fn min(self, rhs: Vec4d) -> Vec4d {
    Vec4d {
      data: self.data.min(rhs.data),
    }
  }
  pub fn max(self, rhs: Vec4d) -> Vec4d {
    Vec4d {
      data: self.data.max(rhs.data),
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
    return Vec4d {
      data: self.data + _rhs.data,
    };
  }
}

impl ops::Sub<Vec4d> for Vec4d {
  type Output = Vec4d;

  fn sub(self, _rhs: Vec4d) -> Vec4d {
    Vec4d {
      data: self.data - _rhs.data,
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
  assert_eq!(Vec4d::vector(2., 0., 0.).normalize(), Vec4d::vector(1., 0., 0.));
}
