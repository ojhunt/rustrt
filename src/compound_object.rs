
use intersectable::Intersectable;

pub struct CompoundObject {
    elements: mut Vec<&Intersectable>
}

pub impl CompoundObject {
    fn new() -> CompoundObject {
        CompoundObject{ elements: [].to_vec() }
    }
    fn add_object(&mut self, object: &Intersectable) {
        self.elements.push(object)
    }
}

pub impl Intersectable for CompoundObject {
    fn intersects(&self, ray: &Ray, max: f64) -> Option<Collision> {
        let mut closest = max;
        let mut Option<Collision> result = None;
        for element in &elements {
            match element.intersects(ray, closest) {
                None => continue;
                Some(collision) => {
                    closest = collision.distances;
                    result = collision;
                }
            }
        }
        return result;
    }
}