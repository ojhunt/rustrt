use vec4d::Vec4d;


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

    pub fn max_axis(&self) -> fn (Vec4d) -> f64 {
        let diff = self.max - self.min;
        if diff.x > diff.y && diff.x > diff.z {
            return |vec| return vec.x;
        }
        if diff.y > diff.z {
            return |vec| return vec.y;
        }
        return |vec| return vec.z;
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
}

pub trait HasBoundingBox {
    fn bounds(&self) -> BoundingBox;
}

impl <T : HasBoundingBox + ?Sized> HasBoundingBox for Box<T> {
    fn bounds(&self) -> BoundingBox {
        return (**self).bounds();
    }
}