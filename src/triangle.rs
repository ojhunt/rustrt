use vec4d::Vec4d;
use ray::Ray;
use collision::Collision;
use bounding_box::{*};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub origin: Vec4d,
    pub edges: [Vec4d; 2]
}

impl Triangle {
    pub fn new(v0: Vec4d, v1: Vec4d, v2: Vec4d) -> Triangle {
        let edge0 = v1 - v0;
        let edge1 = v2 - v0;
        Triangle {
            origin: v0,
            edges: [edge0, edge1],
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let result = BoundingBox::new_from_point(self.origin)
                    .merge_with_point(self.origin + self.edges[0])
                    .merge_with_point(self.origin + self.edges[1]);
        return result;
    }

    pub fn intersects(&self, ray: Ray, max: f64) -> Option<Collision> {
        let h = ray.direction.cross(self.edges[1]);
        let a = self.edges[0].dot(h);
        if a.abs() < 0.00001 {
            return None;
        }
        let f = 1.0 / a;
        let s = ray.origin - self.origin;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None
        }
        let q = s.cross(self.edges[0]);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || (u + v) > 1. {
            return None;
        }
        let t = f * self.edges[1].dot(q);
        if t < 0.00001 || t >= max {
            return None;
        }
        
        return Some(Collision::new(t, (u, v)));
    }
}

impl HasBoundingBox for Triangle {
    fn bounds(&self) -> BoundingBox {
        return self.bounding_box()
    }
}
