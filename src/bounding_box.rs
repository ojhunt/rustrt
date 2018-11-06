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
        return self.merge_with_point(other.min).merge_with_point(other.max);
    }
}

pub trait HasBoundingBox {
    fn bounds(&self) -> BoundingBox;
}
