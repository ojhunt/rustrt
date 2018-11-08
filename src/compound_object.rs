
use intersectable::Intersectable;
use collision::Collision;
use ray::Ray;
use bounding_box::{*};
use bvh::{*};

#[derive(Debug)]
pub struct CompoundObject {
    elements: Vec<Box<Intersectable>>,
    bbox: BoundingBox,
    bvh_tree: Option<Box<BVH>>
}

impl CompoundObject {
    pub fn new() -> CompoundObject {
        CompoundObject{ elements: Vec::new(), bbox: BoundingBox::new(), bvh_tree:None }
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
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision> {
        match self.bvh_tree {
            Some(ref tree) =>  { 
                return tree.intersect(&self.elements, ray, 0.0, max);
            },
            None => panic!()
        }
        

    }
}
