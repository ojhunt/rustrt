
use ray::Ray;
use triangle::Triangle;
use collision::Collision;
use intersectable::Intersectable;
use bounding_box::*;

#[derive(Debug)]
pub struct BasicObject {
    triangles: Vec<Triangle>,
    bbox: BoundingBox
}

fn compute_bounds(triangles: &Vec<Triangle>) -> BoundingBox {
   let mut bounds = triangles[0].bounding_box();
   for triangle in triangles {
       bounds = bounds.merge_with_bbox(triangle.bounding_box())
   }
   return bounds;
}

impl BasicObject {
    pub fn new(triangles: Vec<Triangle>) -> BasicObject {
        let bounds = compute_bounds(&triangles);
        BasicObject{triangles: triangles, bbox: bounds}
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