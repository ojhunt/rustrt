use bounding_box::*;
use collision::Collision;
use intersectable::*;
use ray::Ray;
use vec4d::Vec4d;

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub origin: Vec4d,
    pub edges: [Vec4d; 2],
    pub normals: [Option<usize>; 3],
    pub texture_coords: [Option<usize>; 3],
}

type Vertex = (Vec4d, Option<usize>, Option<usize>);

impl Triangle {
    pub fn new((v0, n0, t0): Vertex, (v1, n1, t1): Vertex, (v2, n2, t2): Vertex) -> Triangle {
        let edge0 = v1 - v0;
        let edge1 = v2 - v0;
        Triangle {
            origin: v0,
            edges: [edge0, edge1],
            normals: [n0, n1, n2],
            texture_coords: [t0, t1, t2],
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let result = BoundingBox::new_from_point(self.origin)
            .merge_with_point(self.origin + self.edges[0])
            .merge_with_point(self.origin + self.edges[1]);
        return result;
    }

    pub fn intersects<'a>(
        &'a self,
        ray: Ray,
        min: f64,
        max: f64,
    ) -> Option<(Collision, &'a Intersectable)> {
        let h = ray.direction.cross(self.edges[1]);
        let a = self.edges[0].dot(h);
        if a.abs() < 0.00001 {
            return None;
        }
        let f = 1.0 / a;
        let s = ray.origin - self.origin;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = s.cross(self.edges[0]);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || (u + v) > 1. {
            return None;
        }
        let t = f * self.edges[1].dot(q);
        if t < min - 0.001 || t >= max {
            return None;
        }

        return Some((Collision::new(t, (u, v)), self));
    }
}

impl HasBoundingBox for Triangle {
    fn bounds(&self) -> BoundingBox {
        return self.bounding_box();
    }
}

impl Intersectable for Triangle {
    fn intersect<'a>(
        &'a self,
        ray: Ray,
        min: f64,
        max: f64,
    ) -> Option<(Collision, &'a Intersectable)> {
        return self.intersects(ray, min, max);
    }
}
