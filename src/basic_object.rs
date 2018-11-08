
use ray::Ray;
use triangle::Triangle;
use collision::Collision;
use intersectable::Intersectable;
use bounding_box::*;
use bvh::BVH;

#[derive(Debug)]
pub struct BasicObject {
    triangles: Vec<Triangle>,
    // tree: Box<BVH>,
    bbox: BoundingBox
}

fn compute_bounds(triangles: &[Triangle]) -> BoundingBox {
   let mut bounds = BoundingBox::new();
   for triangle in triangles {
       let tbounds = triangle.bounding_box();
       bounds = bounds.merge_with_bbox(tbounds);
   }
   return bounds;
}

impl BasicObject {
    pub fn new(triangles: &[Triangle]) -> BasicObject {
        let bounds = compute_bounds(triangles);
        BasicObject{triangles: triangles.to_vec(), bbox: bounds}
    }
}

impl HasBoundingBox for BasicObject {
    fn bounds(&self) -> BoundingBox {
        return self.bbox;
    }
}

impl Intersectable for BasicObject {
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision> {
        let mut result : Option<Collision> = None;
        let mut closest = max;
        for triangle in &self.triangles {
            match triangle.intersects(ray, closest) {
                None => continue,
                Some(collision) => {
                    closest = collision.distance;
                    result = Some(collision);
                }
            }
        }
        return result;
    }
}