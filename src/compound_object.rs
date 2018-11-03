
use intersectable::Intersectable;
use collision::Collision;
use ray::Ray;

#[derive(Debug)]
pub struct CompoundObject {
    pub elements: Vec<Box<Intersectable>>
}

impl CompoundObject {
    pub fn new() -> CompoundObject {
        CompoundObject{ elements: Vec::new() }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self.elements.push(object)
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