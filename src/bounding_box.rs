use vec4d::Vec4d;
use ray::Ray;

#[derive(Debug,Clone,Copy)]
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
        let mut min = self.min;
        let mut max = self.max;
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
        return BoundingBox{min:min, max:max};
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
        let mut t0 = min;
        let mut t1 = max;
        for i in 0..3 {
            let inverse_dir = 1.0 / ray.direction[i];
            let mut tnear = (self.min[i] - ray.origin[i]) * inverse_dir;
            let mut tfar = (self.max[i] - ray.origin[i]) * inverse_dir;
            if tnear > tfar {
                let temp = tnear;
                tnear = tfar;
                tfar = temp;
            }
            t0 = t0.max(tnear);
            t1 = t1.min(tfar);
            if t0 > t1 {
                return None;
            }
        }
        return Some((t0, t1));
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

pub trait HasBoundingBox {
    fn bounds(&self) -> BoundingBox;
}

impl <T : HasBoundingBox + ?Sized> HasBoundingBox for Box<T> {
    fn bounds(&self) -> BoundingBox {
        return (**self).bounds();
    }
}

#[test]
fn test_known_bad() {
    let ray = Ray { 
        origin: Vec4d { x: 0.0, y: 1.0, z: 3.0, w: 1.0 },
        direction: Vec4d { x: -0.40940965317634315, y: 0.40239120197903444, z: -0.8188193063526863, w: 0.0 } 
    };
    let bounds = BoundingBox {
        min: Vec4d { x: -1.0199999809265137, y: -1.0199999809265137, z: -1.0199999809265137, w: 1.0 },
        max: Vec4d { x: 1.0, y: 1.0, z: 1.0, w: 1.0 }
    };
    let mut origin = ray.origin;
    let step = ray.direction.scale(0.01);
    let mut was_interior = false;
    for i in 0..100 {
        if bounds.contains(origin) {
            was_interior = true;
            break;
        }
        origin = origin + step;
    }
    assert!(was_interior);
    assert!(bounds.intersect(ray, 0.0, std::f64::INFINITY).is_some());
}
