use crate::light::Light;
use crate::bounding_box::*;
use crate::bvh::BVH;
use crate::collision::Collision;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::shader::*;

use crate::triangle::Triangle;

#[derive(Debug)]
pub struct Mesh {
  triangles: Vec<Triangle>,
  tree: Box<BVH>,
  bbox: BoundingBox,
}

fn compute_bounds(triangles: &[Triangle]) -> BoundingBox {
  let mut bounds = BoundingBox::new();
  for triangle in triangles {
    let tbounds = triangle.bounding_box();
    bounds = bounds.merge_with_bbox(tbounds);
  }
  return bounds;
}

impl Mesh {
  pub fn new(triangles: &[Triangle]) -> Mesh {
    let bounds = compute_bounds(triangles);
    Mesh {
      triangles: triangles.to_vec(),
      tree: Box::new(BVH::new(triangles)),
      bbox: bounds,
    }
  }
}

impl HasBoundingBox for Mesh {
  fn bounds(&self) -> BoundingBox {
    return self.bbox;
  }
}

impl Intersectable for Mesh {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light> {
    let mut result: Vec<&'a Light> = vec![];
    for triangle in &self.triangles {
      result.append(&mut triangle.get_lights(s));
    }
    return result;
  }

  fn intersect<'a>(&'a self, ray: &Ray, min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
    return self.tree.intersect(&self.triangles, ray, min, max);
  }
}
