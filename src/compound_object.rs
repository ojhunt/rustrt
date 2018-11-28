use bounding_box::*;
use bvh::*;
use collision::Collision;
use intersectable::Intersectable;
use ray::Ray;
use shader::Shadable;

#[derive(Debug)]
pub struct CompoundObject {
    elements: Vec<Box<Intersectable>>,
    bbox: BoundingBox,
    bvh_tree: Option<Box<BVH>>,
}

impl CompoundObject {
    pub fn new() -> CompoundObject {
        CompoundObject {
            elements: Vec::new(),
            bbox: BoundingBox::new(),
            bvh_tree: None,
        }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self.bbox = self.bbox.merge_with_bbox(object.bounds());
        self.elements.push(object);
    }
    pub fn finalize(&mut self) {
        self.bvh_tree = Some(Box::new(BVH::new(&self.elements)))
    }
}

impl HasBoundingBox for CompoundObject {
    fn bounds(&self) -> BoundingBox {
        return self.bbox;
    }
}

impl Intersectable for CompoundObject {
    fn intersect<'a>(
        &'a self,
        ray: &Ray,
        _min: f64,
        max: f64,
    ) -> Option<(Collision, &'a Shadable)> {
        match self.bvh_tree {
            Some(ref tree) => {
                return tree.intersect(&self.elements, ray, ray.min, max);
            }
            None => panic!(),
        }
    }
}
