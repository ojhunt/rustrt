use crate::light::Light;
use crate::bounding_box::*;
use crate::bvh::*;
use crate::collision::Collision;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::shader::*;

#[derive(Debug)]
pub struct CompoundObject {
  elements: Vec<Box<Intersectable>>,
  bbox: BoundingBox,
  bvh_tree: Option<Box<BVH>>,
}

impl CompoundObject {
  pub fn new() -> CompoundObject {
    CompoundObject {
      elements: Vec::new(),
      bbox: BoundingBox::new(),
      bvh_tree: None,
    }
  }

  pub fn add_object(&mut self, object: Box<Intersectable>) {
    self.bbox = self.bbox.merge_with_bbox(object.bounds());
    self.elements.push(object);
  }

  pub fn finalize(&mut self) {
    self.bvh_tree = Some(Box::new(BVH::new(&self.elements)))
  }
}

impl HasBoundingBox for CompoundObject {
  fn bounds(&self) -> BoundingBox {
    return self.bbox;
  }
}

impl Intersectable for CompoundObject {
  fn intersect<'a>(&'a self, ray: &Ray, _min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
    match self.bvh_tree {
      Some(ref tree) => {
        return tree.intersect(&self.elements, ray, ray.min, max);
      }
      None => panic!(),
    }
  }

  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light> {
    let mut result: Vec<&'a Light> = vec![];
    for element in &self.elements {
      result.append(&mut element.get_lights(s));
    }
    return result;
  }
}
