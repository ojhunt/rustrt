use bounding_box::*;
use collision::Collision;
use fragment::Fragment;
use intersectable::*;
use ray::Ray;
use scene::MaterialIdx;
use scene::NormalIdx;
use scene::Scene;
use scene::TextureCoordinateIdx;
use shader::Shadable;
use vectors::{Vec2d, Vec4d};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub material: Option<MaterialIdx>,
    pub origin: Vec4d,
    pub edges: [Vec4d; 2],
    pub normals: [Option<NormalIdx>; 3],
    pub texture_coords: [Option<TextureCoordinateIdx>; 3],
}
type Vertex = (Vec4d, Option<TextureCoordinateIdx>, Option<NormalIdx>);

impl Shadable for Triangle {
    fn compute_fragment(&self, s: &Scene, r: &Ray, collision: Collision) -> Fragment {
        let u = collision.uv.0;
        let v = collision.uv.1;
        let w = 1.0 - u - v;
        let normal: Vec4d = match (self.normals[0], self.normals[1], self.normals[2]) {
            (Some(n_idx0), Some(n_idx1), Some(n_idx2)) => {
                let normal0 = n_idx0.get(s);
                let normal1 = n_idx1.get(s);
                let normal2 = n_idx2.get(s);
                assert!(normal0.dot(normal1) >= 0.0);
                assert!(normal0.dot(normal2) >= 0.0);
                assert!(normal2.dot(normal1) >= 0.0);
                normal0 * w + normal1 * u + normal2 * v
            }
            (Some(idx), None, None) => idx.get(s),
            (None, Some(idx), None) => idx.get(s),
            (None, None, Some(idx)) => idx.get(s),
            _ => self.edges[0]
                .normalize()
                .cross(self.edges[1].normalize())
                .normalize(),
        };
        let mut dpdu = Vec4d::new();
        let mut dpdv = Vec4d::new();
        let mut texture_coords = Vec2d(0.0, 0.0);
        match (
            self.texture_coords[0],
            self.texture_coords[1],
            self.texture_coords[2],
        ) {
            (Some(n_idx0), Some(n_idx1), Some(n_idx2)) => {
                let t0 = n_idx0.get(s);
                let t1 = n_idx1.get(s);
                let t2 = n_idx2.get(s);
                texture_coords = Vec2d(
                    t0.0 * w + t1.0 * u + t2.0 * v,
                    t0.1 * w + t1.1 * u + t2.1 * v,
                );

                let uv_edge0 = t1 - t0;
                let uv_edge1 = t2 - t0;
                let determinant = uv_edge0.0 * uv_edge1.1 - uv_edge0.1 * uv_edge1.0;
                if determinant == 0.0 {
                } else {
                    let edge0 = self.edges[0];
                    let edge1 = self.edges[1];
                    dpdu = (uv_edge1.1 * edge0 - uv_edge0.1 * edge1) * (1.0 / determinant);
                    dpdv = (uv_edge0.0 * edge1 - uv_edge1.0 * edge0) * (1.0 / determinant);
                }
            }
            (Some(idx), None, None) => {
                idx.get(s);
            }
            (None, Some(idx), None) => {
                idx.get(s);
            }
            (None, None, Some(idx)) => {
                idx.get(s);
            }
            _ => {}
        };

        return Fragment {
            position: r.origin + r.direction * collision.distance,
            normal: normal,
            uv: texture_coords,
            dpdu,
            dpdv,
            material: self.material,
        };
    }
}

impl Triangle {
    pub fn new(
        material: Option<MaterialIdx>,
        (v0, t0, n0): Vertex,
        (v1, t1, n1): Vertex,
        (v2, t2, n2): Vertex,
    ) -> Triangle {
        let edge0 = v1 - v0;
        let edge1 = v2 - v0;
        Triangle {
            material,
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
        ray: &Ray,
        min: f64,
        max: f64,
    ) -> Option<(Collision, &'a Shadable)> {
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

        return Some((Collision::new(t, Vec2d(u, v)), self));
    }
}

impl HasBoundingBox for Triangle {
    fn bounds(&self) -> BoundingBox {
        return self.bounding_box();
    }
}

impl Intersectable for Triangle {
    fn intersect<'a>(&'a self, ray: &Ray, min: f64, max: f64) -> Option<(Collision, &'a Shadable)> {
        return self.intersects(ray, min, max);
    }
}
