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

  pub fn max_value(&self) -> f32 {
    let Colour::RGB(r, g, b) = self;
    return r.max(*g).max(*b);
  }
  pub fn r(&self) -> f32 {
    let Colour::RGB(r, _, _) = *self;
    return r;
  }
  pub fn g(&self) -> f32 {
    let Colour::RGB(_, g, _) = *self;
    return g;
  }
  pub fn b(&self) -> f32 {
    let Colour::RGB(_, _, b) = *self;
    return b;
  }
  pub fn new() -> Self {
    return Colour::RGB(0.0, 0.0, 0.0);
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

impl ops::Mul<f32> for Colour {
  type Output = Colour;
  fn mul(self, rhs: f32) -> Colour {
    return self.scale(rhs);
  }
}
impl ops::Mul<Colour> for Colour {
  type Output = Colour;
  fn mul(self, Colour::RGB(rr, rg, rb): Colour) -> Colour {
    let Colour::RGB(r, g, b) = self;
    return Colour::RGB(r * rr, g * rg, b * rb);
  }
}

impl ops::Mul<Colour> for f32 {
  type Output = Colour;
  fn mul(self, rhs: Colour) -> Colour {
    return rhs.scale(self);
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
