use std::ops;
use crate::vectors::Vector;

#[derive(Debug, Copy, Clone)]
pub enum Colour {
  RGB(f32, f32, f32),
}

impl Colour {
  pub fn scale(&self, scale: f32) -> Colour {
    let Colour::RGB(r, g, b) = *self;
    return Colour::RGB(r * scale, g * scale, b * scale);
  }
  pub fn add_elements(&self, Colour::RGB(rr, rg, rb): &Colour) -> Colour {
    let Colour::RGB(r, g, b) = *self;
    return Colour::RGB(r + rr, g + rg, b + rb);
  }

  pub fn max_value(&self) -> f64 {
    let Colour::RGB(r, g, b) = self;
    return r.max(*g).max(*b) as f64;
  }
  pub fn r(&self) -> f64 {
    let Colour::RGB(r, _, _) = *self;
    return r as f64;
  }
  pub fn g(&self) -> f64 {
    let Colour::RGB(_, g, _) = *self;
    return g as f64;
  }
  pub fn b(&self) -> f64 {
    let Colour::RGB(_, _, b) = *self;
    return b as f64;
  }
}

impl From<Colour> for Vector {
  fn from(Colour::RGB(x, y, z): Colour) -> Self {
    Vector::vector(x as f64, y as f64, z as f64)
  }
}

impl From<Vector> for Colour {
  fn from(Vector { data }: Vector) -> Self {
    let mut value = [0.0; 4];
    data.write_to_slice_unaligned(&mut value);
    Colour::RGB(value[0], value[1], value[2])
  }
}

impl ops::Mul<f64> for Colour {
  type Output = Colour;
  fn mul(self, rhs: f64) -> Colour {
    return self.scale(rhs as f32);
  }
}
impl ops::Mul<Colour> for Colour {
  type Output = Colour;
  fn mul(self, Colour::RGB(rr, rg, rb): Colour) -> Colour {
    let Colour::RGB(r, g, b) = self;
    return Colour::RGB(r * rr, g * rg, b * rb);
  }
}

impl ops::Mul<Colour> for f64 {
  type Output = Colour;
  fn mul(self, rhs: Colour) -> Colour {
    return rhs.scale(self as f32);
  }
}

impl ops::Add<Colour> for Colour {
  type Output = Colour;

  fn add(self, rhs: Colour) -> Colour {
    return self.add_elements(&rhs);
  }
}

impl ops::Sub<Colour> for Colour {
  type Output = Colour;

  fn sub(self, rhs: Colour) -> Colour {
    return self + rhs * -1.;
  }
}
