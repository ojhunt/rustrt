
use ray::Ray as Ray;
use collision::Collision;
use compound_object::CompoundObject;
use intersectable::Intersectable;

#[derive(Debug)]
pub struct Scene {
    _scene: CompoundObject
}

impl Scene {
    pub fn new() -> Scene {
        Scene { _scene: CompoundObject::new() }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self._scene.add_object(object)
    }

    pub fn intersect(&self, ray: Ray) -> Option<Collision> {
        return self._scene.intersect(ray, std::f64::INFINITY)
    }
}
