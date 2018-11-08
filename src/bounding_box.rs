use vec4d::Vec4d;
use ray::Ray;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct BoundingBox {
    pub min: Vec4d,
    pub max: Vec4d
}

impl BoundingBox {
    pub fn centroid(&self) -> Vec4d { 
        return self.min.add_elements(self.max).scale(0.5);
    }

    pub fn surface_area(&self) -> f64 {
        let size = self.max - self.min;
        return 2. * (size.x * size.y + size.x * size.z + size.y * size.z);
    }

    pub fn new() -> BoundingBox {
        BoundingBox{
            min:Vec4d::point(std::f64::INFINITY, std::f64::INFINITY, std::f64::INFINITY),
            max:Vec4d::point(-std::f64::INFINITY, -std::f64::INFINITY, -std::f64::INFINITY)
        }
    }

    pub fn is_valid(&self) -> bool {
        let min = self.min;
        let max = self.max;
        
        let valid_values = min.x <= max.x && min.y <= max.y && min.z <= max.z;
        if !valid_values {
            return false;
        }
        return min.x.is_finite() && min.y.is_finite() && min.z.is_finite() &&
               max.x.is_finite() && max.y.is_finite() && max.z.is_finite();
    }

    pub fn new_from_point(v: Vec4d) -> BoundingBox {
        assert!(v.w == 1.);
        BoundingBox{min: v, max: v}
    }

    pub fn merge_with_point(&self, v: Vec4d) -> BoundingBox {
        assert!(v.w == 1.);
        return self.merge_with_bbox(BoundingBox{min:v, max:v});
        
    }
    pub fn merge_with_bbox(&self, other: BoundingBox) -> BoundingBox {
        return BoundingBox{
            min:self.min.min(other.min),
            max:self.max.max(other.max),
        };
    }

    pub fn max_axis(&self) -> usize {
        let diff = self.max - self.min;
        if diff.x > diff.y && diff.x > diff.z {
            return 0;
        }
        if diff.y > diff.z {
            return 1;
        }
        return 2;
    }

    pub fn offset(&self, point: Vec4d) -> Vec4d {
        let mut o = point - self.min;
        if self.max.x > self.min.x {
            o.x /= self.max.x - self.min.x;
        }
        if self.max.x > self.min.x {
            o.y /= self.max.y - self.min.y;
        }
        if self.max.x > self.min.x {
            o.z /= self.max.z - self.min.z;
        }
        return o;
    }

    pub fn intersect(&self, ray: Ray, min: f64, max: f64) -> Option<(f64, f64)> {
        let mut tmin = min;
        let mut tmax = max;
        let direction = ray.direction;
        let origin = ray.origin;
        for i in 0..3 {
            if direction[i].abs() < std::f64::EPSILON {
                if origin[i] < self.min[i] || origin[i] > self.max[i] {
                    return None;
                }
                continue;
            }
            let inverse_dir = 1.0 / direction[i];
            
            let mut t1 = (self.min[i] - origin[i]) * inverse_dir;
            let mut t2 = (self.max[i] - origin[i]) * inverse_dir;

            if t1 > t2 {
                let temp = t1;
                t1 = t2;
                t2 = temp;
            }
            // tmin *= 1. + 2. * gamma(3);
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
        return Some((tmin, tmax));
    }

    pub fn contains(&self, point: Vec4d) -> bool {
        if point.x < self.min.x || point.y < self.min.y || point.z < self.min.z  {
            return false;
        }
        if point.x > self.max.x || point.y > self.max.y || point.z > self.max.z  {
            return false;
        }
        return true;
    }

    pub fn encloses(&self, other: BoundingBox) -> bool {
        for i in 0..3 {
            if other.min[i] > self.max[i] || other.min[i] < self.min[i] {
                return false;
            }
            if other.max[i] > self.max[i] || other.max[i] < self.min[i] {
                return false;
            }
        }
        return true;
    }
}

const MACHINE_EPSILON : f64 = std::f64::EPSILON * 0.5;
fn gamma(value: i64) -> f64 {
    return value as f64 * MACHINE_EPSILON / ((1 - value) as f64 * MACHINE_EPSILON);
}

pub trait HasBoundingBox {
    fn bounds(&self) -> BoundingBox;
}

impl <T : HasBoundingBox + ?Sized> HasBoundingBox for Box<T> {
    fn bounds(&self) -> BoundingBox {
        return (**self).bounds();
    }
}

