use std::ops;
use packed_simd::shuffle;
use packed_simd::f32x4;
use packed_simd::m32x4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
  pub data: f32x4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
  pub data: f32x4,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4Mask(m32x4);

pub trait VectorType: Sized + Copy {
  fn data(self) -> f32x4;
  fn new(data: f32x4) -> Self;

  fn x(&self) -> f32 {
    self.data().extract(0)
  }
  fn y(&self) -> f32 {
    self.data().extract(1)
  }
  fn z(&self) -> f32 {
    self.data().extract(2)
  }
  fn w(&self) -> f32 {
    self.data().extract(3)
  }
  fn splat(value: f32) -> Self {
    Self::new(f32x4::splat(value))
  }

  fn fdiv(self, divisor: f32) -> Self {
    Self::new(self.data() / divisor)
  }
  fn scale(self, scale: f64) -> Self {
    Self::new(self.data() * scale as f32)
  }
  fn divide_elements(self, _rhs: Self) -> Self {
    Self::new(self.data() / _rhs.data())
  }

  fn multiply_elements(self, _rhs: Self) -> Self {
    Self::new(self.data() * _rhs.data())
  }

  fn gt(self, other: Self) -> Vec4Mask {
    Vec4Mask(self.data().gt(other.data()))
  }
  fn lt(self, other: Self) -> Vec4Mask {
    Vec4Mask(self.data().lt(other.data()))
  }

  fn add_elements(self, _rhs: Self) -> Self {
    Self::new(self.data() + _rhs.data())
  }
  fn min(self, rhs: Self) -> Self {
    Self::new(self.data().min(rhs.data()))
  }
  fn max(self, rhs: Self) -> Self {
    Self::new(self.data().max(rhs.data()))
  }
  fn max_element(self) -> f32 {
    self.data().max_element()
  }
  fn min_element(self) -> f32 {
    self.data().min_element()
  }
}

impl Vec4Mask {
  pub fn select<T: VectorType>(self, left: T, right: T) -> T {
    T::new(self.0.select(left.data(), right.data()))
  }
  pub fn any(self) -> bool {
    self.0.any()
  }
}

impl VectorType for Vector {
  fn data(self) -> f32x4 {
    return self.data;
  }
  fn new(data: f32x4) -> Self {
    return Vector { data };
  }
}

impl Vector {
  pub fn new() -> Vector {
    Vector {
      data: f32x4::splat(0.0),
    }
  }

  pub fn vector(x: f64, y: f64, z: f64) -> Vector {
    Vector {
      data: f32x4::new(x as f32, y as f32, z as f32, 0.0),
    }
  }

  pub fn point(x: f64, y: f64, z: f64) -> Point {
    Point {
      data: f32x4::new(x as f32, y as f32, z as f32, 1.0),
    }
  }

  pub fn reflect(&self, normal: Vector) -> Vector {
    assert!(self.dot(normal) <= 0.0);
    (-2.0 * self.dot(normal) * normal + *self).normalize()
  }

  pub fn cross(&self, rhs: Vector) -> Vector {
    let rhs201: f32x4 = shuffle!(rhs.data, [2, 0, 1, 0]);
    let rhs120: f32x4 = shuffle!(rhs.data, [1, 2, 0, 0]);
    let lhs120: f32x4 = shuffle!(self.data, [1, 2, 0, 0]);
    let lhs201: f32x4 = shuffle!(self.data, [2, 0, 1, 0]);

    Vector {
      data: lhs120 * rhs201 - lhs201 * rhs120,
    }
  }
  pub fn square_length(&self) -> f64 {
    return self.dot(*self);
  }
  pub fn length(&self) -> f64 {
    return self.square_length().sqrt();
  }
  pub fn dot(&self, rhs: Vector) -> f64 {
    let scaled = self.data * rhs.data;
    return scaled.sum() as f64;
  }
  pub fn normalize(&self) -> Vector {
    let scale = 1.0 / self.dot(*self).sqrt();
    return *self * scale;
  }
}

impl ops::Neg for Vector {
  type Output = Vector;
  fn neg(self) -> Vector {
    return self.scale(-1.0);
  }
}

impl ops::Mul<f64> for Vector {
  type Output = Vector;
  fn mul(self, rhs: f64) -> Vector {
    return self.scale(rhs);
  }
}

impl ops::Mul<Vector> for f64 {
  type Output = Vector;
  fn mul(self, rhs: Vector) -> Vector {
    return rhs.scale(self);
  }
}

impl ops::Div<f64> for Vector {
  type Output = Vector;
  fn div(self, rhs: f64) -> Vector {
    return self.fdiv(rhs as f32);
  }
}

impl ops::Div<f32> for Vector {
  type Output = Vector;
  fn div(self, rhs: f32) -> Vector {
    return self.fdiv(rhs);
  }
}

impl ops::Div<Vector> for Vector {
  type Output = Vector;
  fn div(self, rhs: Vector) -> Vector {
    return self.divide_elements(rhs);
  }
}
impl ops::Mul<Vector> for Vector {
  type Output = Vector;
  fn mul(self, rhs: Vector) -> Vector {
    return self.multiply_elements(rhs);
  }
}

impl ops::Add<Vector> for Vector {
  type Output = Vector;

  fn add(self, _rhs: Vector) -> Vector {
    return Vector {
      data: self.data + _rhs.data,
    };
  }
}

impl ops::Sub<Vector> for Vector {
  type Output = Vector;

  fn sub(self, _rhs: Vector) -> Vector {
    Vector {
      data: self.data - _rhs.data,
    }
  }
}

impl ops::Add<Vector> for Point {
  type Output = Point;

  fn add(self, _rhs: Vector) -> Point {
    return Point {
      data: self.data + _rhs.data,
    };
  }
}
impl ops::Add<Point> for Vector {
  type Output = Point;

  fn add(self, _rhs: Point) -> Point {
    return Point {
      data: self.data + _rhs.data,
    };
  }
}

impl ops::Sub<Vector> for Point {
  type Output = Point;

  fn sub(self, _rhs: Vector) -> Point {
    Point {
      data: self.data - _rhs.data,
    }
  }
}

impl ops::Sub<Point> for Point {
  type Output = Vector;

  fn sub(self, _rhs: Point) -> Vector {
    Vector {
      data: self.data - _rhs.data,
    }
  }
}

impl VectorType for Point {
  fn new(data: f32x4) -> Self {
    Point { data }
  }
  fn data(self) -> f32x4 {
    self.data
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
  assert_eq!(Vector::vector(0., 1., 0.).dot(Vector::vector(1., 0., 0.)), 0.);
}
#[test]
fn test_cross() {
  assert_eq!(
    Vector::vector(2., 1., -1.).cross(Vector::vector(-3., 4., 1.)),
    Vector::vector(5., 1., 11.)
  );
  assert_eq!(
    Vector::vector(-3., 4., 1.).cross(Vector::vector(2., 1., -1.)),
    Vector::vector(-5., -1., -11.)
  );
}

#[test]
fn test_normalize() {
  assert_eq!(Vector::vector(2., 0., 0.).normalize(), Vector::vector(1., 0., 0.));
}
