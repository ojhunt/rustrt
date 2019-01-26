use scene::MaterialIdx;
use vectors::Vec2d;
use vectors::*;

pub struct Fragment {
  pub position: Point,
  pub normal: Vec4d,
  pub true_normal: Vec4d,
  pub uv: Vec2d,

  pub dpdu: Vec4d,
  pub dpdv: Vec4d,

  pub view: Vec4d, // Camera -> Point

  pub material: MaterialIdx,
}
