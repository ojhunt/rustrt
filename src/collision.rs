use crate::vectors::Vec2d;

#[derive(PartialEq, Debug)]
pub struct Collision {
  pub distance: f64,
  pub uv: Vec2d,
  pub intersection_count: usize,
  pub node_count: usize,
}

impl Collision {
  pub fn new(distance: f64, uv: Vec2d) -> Collision {
    Collision {
      distance,
      uv,
      intersection_count: 0,
      node_count: 0,
    }
  }
}
