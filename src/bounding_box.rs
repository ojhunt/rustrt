use vec4d::Vec4d;


#[derive(Debug,Clone,Copy)]
pub struct BoundingBox {
    pub min: Vec4d,
    pub max: Vec4d
}

impl BoundingBox {
    pub fn new() -> BoundingBox {
        BoundingBox{min:Vec4d::new(), max:Vec4d::new()}
    }
    pub fn new_from_point(v: Vec4d) -> BoundingBox {
        assert!(v.w == 1.);
        BoundingBox{min: v, max: v}
    }

    pub fn merge_with_point(&self, v: Vec4d) -> BoundingBox {
        assert!(v.w == 1.);
        let mut min = self.min;
        let mut max = self.max;
        if v.x < min.x { min.x = v.x; }
        if v.y < min.y { min.y = v.y; }
        if v.z < min.z { min.z = v.z; }
        if v.x > max.x { max.x = v.x; }
        if v.y > max.y { max.y = v.y; }
        if v.z > max.z { max.z = v.z; }
        return BoundingBox{min:min, max:max};
    }
    pub fn merge_with_bbox(&self, other: BoundingBox) -> BoundingBox {
        return self.merge_with_point(other.min).merge_with_point(other.max);
    }
}

pub trait HasBoundingBox {
    fn bounds(&self) -> BoundingBox;
}
