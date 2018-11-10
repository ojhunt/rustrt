use bounding_box::*;
use bvh::BVH;
use collision::Collision;
use intersectable::Intersectable;
use ray::Ray;
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
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision> {
        return self.tree.intersect(&self.triangles, ray, 0.0, max);
    }
}
