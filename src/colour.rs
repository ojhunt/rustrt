use std::ops;
use vectors::Vec4d;

#[derive(Debug, Copy, Clone)]
pub enum Colour {
  RGB(f64, f64, f64),
}

impl Colour {
  pub fn scale(&self, scale: f64) -> Colour {
    let Colour::RGB(r, g, b) = *self;
    return Colour::RGB(r * scale, g * scale, b * scale);
  }
  pub fn add_elements(&self, Colour::RGB(rr, rg, rb): &Colour) -> Colour {
    let Colour::RGB(r, g, b) = *self;
    return Colour::RGB(r + rr, g + rg, b + rb);
  }

  pub fn max_value(&self) -> f64 {
    let Colour::RGB(r, g, b) = self;
    return r.max(*g).max(*b);
  }
}

impl From<Colour> for Vec4d {
  fn from(Colour::RGB(x, y, z): Colour) -> Self {
    Vec4d::vector(x, y, z)
  }
}

impl From<Vec4d> for Colour {
  fn from(
    Vec4d {
      x: r,
      y: g,
      z: b,
      w: _a,
    }: Vec4d
  ) -> Self {
    Colour::RGB(r, g, b)
  }
}

impl ops::Mul<f64> for Colour {
  type Output = Colour;
  fn mul(self, rhs: f64) -> Colour {
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

impl ops::Mul<Colour> for f64 {
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
