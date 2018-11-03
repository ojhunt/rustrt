
use ray::Ray;
use triangle::Triangle;
use collision::Collision;
use intersectable::Intersectable;

#[derive(Debug)]
pub struct BasicObject {
    triangles: Vec<Triangle>
}

impl BasicObject {
    pub fn new(triangles: Vec<Triangle>) -> BasicObject {
        BasicObject{triangles: triangles}
    }
}

impl Intersectable for BasicObject {
    fn intersect(&self, ray: Ray, max: f64) -> Option<Collision> {
        let mut result : Option<Collision> = None;
        let mut closest = max;
        for triangle in &self.triangles {
            match triangle.intersects(ray, closest) {
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