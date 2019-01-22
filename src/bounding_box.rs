use ray::Ray;
use vectors::Vec4d;
use faster::arch::x86::vecs::f32x4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
  pub min: Vec4d,
  pub max: Vec4d,
}

impl BoundingBox {
  pub fn centroid(&self) -> Vec4d {
    return self.min.add_elements(self.max).scale(0.5);
  }

  pub fn surface_area(&self) -> f64 {
    let size = self.max - self.min;
    return 2. * (size.x() * size.y() + size.x() * size.z() + size.y() * size.z()) as f64;
  }

  pub fn new() -> BoundingBox {
    BoundingBox {
      min: Vec4d::point(std::f64::INFINITY, std::f64::INFINITY, std::f64::INFINITY),
      max: Vec4d::point(-std::f64::INFINITY, -std::f64::INFINITY, -std::f64::INFINITY),
    }
  }

  pub fn is_valid(&self) -> bool {
    let min = self.min;
    let max = self.max;

    let valid_values = min.x() <= max.x() && min.y() <= max.y() && min.z() <= max.z();
    if !valid_values {
      return false;
    }

    return min.x().is_finite()
      && min.y().is_finite()
      && min.z().is_finite()
      && max.x().is_finite()
      && max.y().is_finite()
      && max.z().is_finite();
  }

  pub fn new_from_point(v: Vec4d) -> BoundingBox {
    assert!(v.w() == 1.);
    BoundingBox { min: v, max: v }
  }

  pub fn merge_with_point(&self, v: Vec4d) -> BoundingBox {
    assert!(v.w() == 1.);
    return self.merge_with_bbox(BoundingBox { min: v, max: v });
  }
  pub fn merge_with_bbox(&self, other: BoundingBox) -> BoundingBox {
    return BoundingBox {
      min: self.min.min(other.min),
      max: self.max.max(other.max),
    };
  }

  pub fn max_axis(&self) -> usize {
    let diff = self.max - self.min;
    if diff.x() > diff.y() && diff.x() > diff.z() {
      return 0;
    }
    if diff.y() > diff.z() {
      return 1;
    }
    return 2;
  }

  pub fn offset(&self, point: Vec4d) -> Vec4d {
    let o = point - self.min;
    let mask = self.max.data.gt(self.min.data);
    let scale_factor = self.max.data - self.min.data;
    return Vec4d {
      data: mask.select(o.data / scale_factor, o.data),
    };
  }

  pub fn intersect(&self, ray: &Ray, min: f64, max: f64) -> Option<(f64, f64)> {
    let mut tmin = f32x4::splat(min as f32);
    let mut tmax = f32x4::splat(max as f32);

    let direction = ray.direction;
    let origin = ray.origin;

    let inverse_dir = f32x4::splat(1.0) / direction.data;
    let unnormalized_t1 = (self.min.data - origin.data) * inverse_dir;
    let unnormalized_t2 = (self.max.data - origin.data) * inverse_dir;
    let compare_mask = unnormalized_t1.gt(unnormalized_t2);
    let t1 = compare_mask.select(unnormalized_t2, unnormalized_t1);
    let t2 = compare_mask.select(unnormalized_t1, unnormalized_t2);
    tmin = tmin.max(t1);
    tmax = tmax.min(t2);

    if tmin.gt(tmax).any() {
      return None;
    }
    return Some((tmin.max_element() as f64, (tmax.min_element() + 0.01) as f64));
  }
}

pub trait HasBoundingBox {
  fn bounds(&self) -> BoundingBox;
}

impl<T: HasBoundingBox + ?Sized> HasBoundingBox for Box<T> {
  fn bounds(&self) -> BoundingBox {
    return (**self).bounds();
  }
}
