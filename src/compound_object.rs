
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
    
        let mut closest = max;
        let mut result: Option<Collision> = None;
        let elements = &self.elements;
        for element in elements.iter() {
            match element.intersect(ray, closest) {
                None => continue,
                Some(collision) => {
                    closest = collision.distance;
                    result = Some(collision);
                }
            }
        }

        match self.bvh_tree {
            Some(ref tree) =>  { 
                if tree.intersect(elements, ray, 0.0, max).is_some() != result.is_some() {
                    println!("ray: {:?}, own bounds: {:?}", ray, self.bounds());
                    assert!(self.bbox.intersect(ray,0.,max).is_some());
                    panic!();
                }
            },
            None => panic!()
        }
        
        return result;

    }
}
