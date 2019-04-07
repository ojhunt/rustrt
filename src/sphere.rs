use crate::fragment::Fragment;
use crate::photon_map::random;
use crate::light::LightSample;
use crate::shader::Shadable;
use crate::collision::Collision;
use crate::ray::Ray;
use crate::scene::MaterialIdx;
use crate::light::Light;
use crate::scene::Scene;
use crate::bounding_box::BoundingBox;
use crate::bounding_box::HasBoundingBox;
use crate::vectors::Point;
use crate::intersectable::*;
use crate::vectors::Vector;
use crate::vectors::VectorType;
use crate::vectors::Vec2d;

#[derive(Debug)]
pub struct Sphere {
  position: Point,
  radius: f32,
  material: MaterialIdx,
}

impl HasBoundingBox for Sphere {
  fn bounds(&self) -> BoundingBox {
    let radius_vector = Vector::splat(self.radius);
    return BoundingBox {
      min: self.position - radius_vector,
      max: self.position + radius_vector,
    };
  }
}

impl Sphere {
  #[allow(dead_code)]
  pub fn new(position: Point, radius: f32, material: MaterialIdx) -> Self {
    return Sphere {
      position,
      radius,
      material,
    };
  }
  #[allow(warnings)]
  fn intersects<'a>(&'a self, ray: &Ray, min: f32, max: f32) -> Option<Collision> {
    let origin_to_center = self.position - ray.origin;
    let distance_to_tangent_point = origin_to_center.dot(ray.direction);

    let tangent_point = ray.origin + ray.direction * distance_to_tangent_point;
    let to_tangent_point = tangent_point - self.position;
    let radius_at_tangent_point = to_tangent_point.length();
    if radius_at_tangent_point > self.radius {
      return None;
    }

    let collision_to_tangent = (self.radius * self.radius - radius_at_tangent_point * radius_at_tangent_point).sqrt();

    let true_distance = if origin_to_center.length() < self.radius {
      distance_to_tangent_point + collision_to_tangent
    } else {
      distance_to_tangent_point - collision_to_tangent
    };

    if true_distance < min {
      return None;
    }

    if true_distance >= max {
      return None;
    }

    let collision_point = ray.origin + ray.direction * true_distance;
    assert!(collision_point.is_finite());

    let normal = (collision_point - self.position).normalize();
    assert!(normal.is_finite());
    let u = normal.z().atan2(normal.x());
    let v = normal.y().acos();
    return Some(Collision::new(true_distance, Vec2d(u.into(), v.into())));
  }
}
impl Shadable for Sphere {
  #[allow(warnings)]
  fn compute_fragment(&self, _: &Scene, ray: &Ray, collision: &Collision) -> Fragment {
    let position = ray.origin + collision.distance * ray.direction;
    let c_to_c = position - self.position;
    let internal = c_to_c.length() + 0.0001 < self.radius;
    let normal = (c_to_c).normalize();
    let u = normal.z().atan2(normal.x());
    let v = normal.y().acos();
    let dpdv = normal.cross(Vector::vector(0.0, 1.0, 0.0)).cross(normal);
    let dpdu = normal.cross(Vector::vector(1.0, 0.0, 0.0)).cross(normal);
    return Fragment {
      material: self.material,
      normal: if ray.direction.dot(normal) > 0.0 {
        -normal
      } else {
        normal
      },
      position,
      true_normal: if internal { normal } else { -normal },
      uv: Vec2d(u.into(), v.into()),
      dpdv,
      dpdu,
      view: ray.direction,
    };
  }
}
impl Light for Sphere {
  fn get_area(&self) -> f32 {
    return 4.0 * 3.1412 * self.radius * self.radius;
  }

  fn get_samples(&self, count: usize, scene: &Scene) -> Vec<LightSample> {
    let mut result = vec![];
    while result.len() < count {
      let light_dir = {
        let u = random(0.0, 1.0);
        let v = 2.0 * 3.14127 * random(0.0, 1.0);
        Vector::vector(v.cos() * u.sqrt(), -(1.0 - u).sqrt(), v.sin() * u.sqrt())
      };

      let position = self.position + light_dir * self.radius;

      let ray = Ray::new(position + light_dir, -light_dir, None);
      let collision = self.intersects(&ray, 0.0, std::f32::INFINITY).unwrap();
      let fragment = self.compute_fragment(scene, &ray, &collision);

      let material = scene.get_material(fragment.material);
      let surface = material.compute_surface_properties(scene, &ray, &fragment);

      result.push(LightSample {
        position: position,
        direction: Some(fragment.normal),
        specular: Vector::from(surface.specular_colour),
        diffuse: Vector::from(surface.diffuse_colour),
        ambient: Vector::from(surface.ambient_colour),
        emission: surface.emissive_colour.unwrap(),
        weight: (1.0 / count as f32),
        power: 1.0,
      });
    }
    return result;
  }
}

impl Intersectable for Sphere {
  fn get_lights<'a>(&'a self, _s: &Scene) -> Vec<&'a Light> {
    // if s.get_material(self.material).is_light() {
    //   return vec![self];
    // }
    return vec![];
  }
  fn intersect<'a>(&'a self, ray: &Ray, _: HitMode, min: f32, max: f32) -> Option<(Collision, &'a Shadable)> {
    return self.intersects(ray, min, max).map(|c| (c, self as &Shadable));
  }
}
