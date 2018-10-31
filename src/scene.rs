use triangle::Triangle as Triangle;
use ray::Ray as Ray;
use std::vec::Vec;
use std::option::Option;

#[derive(Debug)]
pub struct Scene {
    elements: Vec<Triangle>
}

impl Scene {
    pub fn new() -> Scene {
        Scene { elements: Vec::new() }
    }
    pub fn addTriangle(&mut self, triangle: &Triangle) {
        self.elements.push(*triangle)
    }

    pub fn intersect(&self, ray: Ray) -> Option<(&Triangle, f64, f64, f64)> {
        let mut hit: Option<&Triangle> = None;
        let mut nearest = std::f64::INFINITY;
        let mut uv : (f64, f64) = (0., 0.);
        for element in &self.elements {
            let (d, u, v) = element.intersects(ray);
            if d < 0. || d > nearest {
                continue;
            }
            nearest = d;
            hit = Some(element);
            uv = (u, v);
        }
        match hit {
            None => return None,
            Some(target) => return Some((target, nearest, uv.0, uv.1))
        }
    }
}
