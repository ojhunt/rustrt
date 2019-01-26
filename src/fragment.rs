use scene::MaterialIdx;
use vectors::Vec2d;
use vectors::*;

pub struct Fragment {
  pub position: Point,
  pub normal: Vector,
  pub true_normal: Vector,
  pub uv: Vec2d,

  pub dpdu: Vector,
  pub dpdv: Vector,

  pub view: Vector, // Camera -> Point

  pub material: MaterialIdx,
}
