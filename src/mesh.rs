use bounding_box::*;
use bvh::BVH;
use collision::Collision;
use intersectable::Intersectable;
use ray::Ray;
use scene::Scene;
use shader::*;

use triangle::Triangle;

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

    fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)> {
        return self.tree.intersect(&self.triangles, ray, min, max);
    }
}
