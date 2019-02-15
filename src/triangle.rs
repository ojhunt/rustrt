use crate::light::Light;
use crate::light::LightSample;
use crate::photon_map::random;
use crate::bounding_box::*;
use crate::collision::Collision;
use crate::fragment::Fragment;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::scene::MaterialIdx;
use crate::scene::NormalIdx;
use crate::scene::Scene;
use crate::shader::*;
use crate::texture::TextureCoordinateIdx;
use crate::vectors::{Point, Vec2d, Vector, VectorType};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
  pub material: MaterialIdx,
  pub origin: Point,
  pub edges: [Vector; 2],
  pub normals: [Option<NormalIdx>; 3],
  pub texture_coords: [Option<TextureCoordinateIdx>; 3],
}
type Vertex = (Point, Option<TextureCoordinateIdx>, Option<NormalIdx>);

fn orient_normal(normal: Vector, ray_direction: Vector) -> Vector {
  if normal.dot(ray_direction) > 0.0 {
    -normal
  } else {
    normal
  }
}

impl Light for Triangle {
  fn get_area(&self) -> f32 {
    return self.edges[0].cross(self.edges[1]).length() / 2.0;
  }
  fn get_samples(&self, count: usize, scene: &Scene) -> Vec<LightSample> {
    let mut lights: Vec<LightSample> = vec![];
    while lights.len() < count {
      let r1 = random(0.0, 1.0) as f32;
      let r1_root = r1.sqrt();
      let r2 = random(0.0, 1.0) as f32;
      let a = self.origin;
      let b = self.origin + self.edges[0];
      let c = self.origin + self.edges[1];
      let point = a
        .scale32(1.0 - r1_root)
        .add_elements(b.scale32(r1_root * (1.0 - r2)))
        .add_elements(c.scale32(r1_root * r2));
      let normal = self.true_normal();
      let (ray, collision) = {
        let ray = Ray::new(point + normal, normal * -1.0, None);
        if let Some((collision, _)) = self.intersect(&ray, 0.0, std::f32::INFINITY) {
          (ray, collision)
        } else {
          let ray = Ray::new(point + normal, normal, None);
          if let Some((collision, _)) = self.intersect(&ray, 0.0, std::f32::INFINITY) {
            (ray, collision)
          } else {
            continue;
          }
        }
      };
      let fragment = self.compute_fragment(scene, &ray, &collision);

      let material = scene.get_material(fragment.material);
      let surface = material.compute_surface_properties(scene, &ray, &fragment);

      let sample = LightSample {
        position: point,
        direction: Some(fragment.normal),
        specular: Vector::from(surface.specular_colour),
        diffuse: Vector::from(surface.diffuse_colour),
        ambient: Vector::from(surface.ambient_colour),
        emission: surface.emissive_colour.unwrap(),
        weight: (1.0 / count as f32),
        power: 1.0,
      };
      lights.push(sample);
    }
    return lights;
  }
}

impl Shadable for Triangle {
  fn compute_fragment(&self, s: &Scene, r: &Ray, collision: &Collision) -> Fragment {
    let u = collision.uv.0;
    let v = collision.uv.1;
    let w = 1.0 - u - v;
    let true_normal: Vector = match (self.normals[0], self.normals[1], self.normals[2]) {
      (Some(n_idx0), Some(n_idx1), Some(n_idx2)) => {
        let true_normal = self.edges[0].normalize().cross(self.edges[1].normalize()).normalize();
        let normal0 = orient_normal(n_idx0.get(s), true_normal);
        let normal1 = orient_normal(n_idx1.get(s), true_normal);
        let normal2 = orient_normal(n_idx2.get(s), true_normal);
        // assert!(normal0.dot(normal1) >= 0.0);
        // assert!(normal0.dot(normal2) >= 0.0);
        // assert!(normal2.dot(normal1) >= 0.0);
        normal0 * w + normal1 * u + normal2 * v
      }
      (Some(idx), None, None) => idx.get(s),
      (None, Some(idx), None) => idx.get(s),
      (None, None, Some(idx)) => idx.get(s),
      _ => self.edges[0].normalize().cross(self.edges[1].normalize()).normalize(),
    };
    let normal = orient_normal(true_normal, r.direction);
    let mut dpdu = Vector::new();
    let mut dpdv = Vector::new();
    let mut texture_coords = Vec2d(0.0, 0.0);
    match (self.texture_coords[0], self.texture_coords[1], self.texture_coords[2]) {
      (Some(n_idx0), Some(n_idx1), Some(n_idx2)) => {
        let t0 = n_idx0.get(s);
        let t1 = n_idx1.get(s);
        let t2 = n_idx2.get(s);
        texture_coords = Vec2d(t0.0 * w + t1.0 * u + t2.0 * v, t0.1 * w + t1.1 * u + t2.1 * v);

        let uv_edge0 = t1 - t0;
        let uv_edge1 = t2 - t0;
        let determinant = uv_edge0.0 * uv_edge1.1 - uv_edge0.1 * uv_edge1.0;
        if determinant == 0.0 {
          let uv_edge0 = t0 - t1;
          let uv_edge1 = t2 - t1;
          let determinant = uv_edge0.0 * uv_edge1.1 - uv_edge0.1 * uv_edge1.0;
          if determinant != 0. {
            let edge0 = -self.edges[0];
            let edge1 = self.edges[1] - self.edges[0];
            dpdu = ((uv_edge1.1 * edge0 - uv_edge0.1 * edge1) * (1.0 / determinant)).normalize();
            dpdv = ((uv_edge0.0 * edge1 - uv_edge1.0 * edge0) * (1.0 / determinant)).normalize();
          } else {
            let uv_edge0 = t0 - t2;
            let uv_edge1 = t1 - t2;
            let edge0 = -self.edges[1];
            let edge1 = self.edges[0] - self.edges[1];
            let determinant = uv_edge0.0 * uv_edge1.1 - uv_edge0.1 * uv_edge1.0;
            if determinant != 0.0 {
              dpdu = ((uv_edge1.1 * edge0 - uv_edge0.1 * edge1) * (1.0 / determinant)).normalize();
              dpdv = ((uv_edge0.0 * edge1 - uv_edge1.0 * edge0) * (1.0 / determinant)).normalize();
            }
          }
        } else {
          let edge0 = self.edges[0];
          let edge1 = self.edges[1];
          dpdu = ((uv_edge1.1 * edge0 - uv_edge0.1 * edge1) * (1.0 / determinant)).normalize();
          dpdv = ((uv_edge0.0 * edge1 - uv_edge1.0 * edge0) * (1.0 / determinant)).normalize();
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
      true_normal: self.edges[1].normalize().cross(self.edges[0].normalize()),
      dpdu: dpdu,
      dpdv: dpdv,
      view: r.direction,
      material: self.material,
    };
  }
}

impl Triangle {
  pub fn new(material: MaterialIdx, (v0, t0, n0): Vertex, (v1, t1, n1): Vertex, (v2, t2, n2): Vertex) -> Triangle {
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

  pub fn intersects<'a>(&'a self, ray: &Ray, min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
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

    return Some((Collision::new(t, Vec2d(u.into(), v.into())), self));
  }

  fn true_normal(&self) -> Vector {
    self.edges[0].normalize().cross(self.edges[1].normalize()).normalize()
  }
}

impl HasBoundingBox for Triangle {
  fn bounds(&self) -> BoundingBox {
    return self.bounding_box();
  }
}

impl Intersectable for Triangle {
  fn get_lights<'a>(&'a self, s: &Scene) -> Vec<&'a Light> {
    if s.get_material(self.material).is_light() {
      return vec![self];
    }
    return vec![];
  }
  fn intersect<'a>(&'a self, ray: &Ray, min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
    return self.intersects(ray, min, max);
  }
}
