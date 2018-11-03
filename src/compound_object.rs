
use intersectable::Intersectable;
use collision::Collision;
use ray::Ray;
use bounding_box::{*};

#[derive(Debug)]
pub struct CompoundObject {
    pub elements: Vec<Box<Intersectable>>,
    bbox: BoundingBox
}

impl CompoundObject {
    pub fn new() -> CompoundObject {
        CompoundObject{ elements: Vec::new(), bbox: BoundingBox::new() }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        if self.elements.len() == 0 {
            self.bbox = object.bounds();
        } else {
            self.bbox = self.bbox.merge_with_bbox(object.bounds());
        }
        self.elements.push(object);
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
        for element in &self.elements {
            match element.intersect(ray, closest) {
                None => continue,
                Some(collision) => {
                    closest = collision.distance;
                    result = Some(collision);
                }
            }
        }
        return result;
    }
}