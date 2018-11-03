use vec4d::Vec4d;
use ray::Ray;
use triangle::Triangle;
use collision::Collision;
use intersectable::Intersectable;

pub struct BasicObject {
    triangles: &Vec<Triangle>
}

impl BasicObject {
    fn new(triangles: &Vec<Triangle>) -> BasicObject {
        BasicObject{triangles: triangles}
    }
}

impl Intersectable for BasicObject {
    fn intersects(&self, ray: &Ray, max: f64) -> Option<Collision> {
        let mut result : Option<Collision> = None;
        let mut closest = max;
        for triangle in triangles {
            match triangle.intersects(ray, closest) {
                None => continue;
                Some(collision) => {
                    closest = collision.distance;
                    result = collision;
                }
            }
        }
        return result;
    }
}