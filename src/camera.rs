use collision::Collision;
use image::*;
use ray::Ray;
use scene::Scene;
use vec4d::Vec4d;

pub struct Camera {
    position: Vec4d,
    direction: Vec4d,
    up: Vec4d,
    fov: f64,
    sensor_size: f64,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec4d::new(),
            direction: Vec4d::new(),
            up: Vec4d::new(),
            fov: 0.0,
            sensor_size: 3.,
        }
    }
    pub fn goto(&mut self, position: Vec4d) {
        self.position = position;
    }
    pub fn lookAt(&mut self, location: Vec4d) {
        self.direction = (location - self.position).normalize();
    }
    pub fn get_ray(&self, xp: f64, yp: f64) -> Ray {
        let origin = Vec4d::point(10., 2., 0.);
        let zdirection = 10. * xp;
        let ydirection = 10. * yp;
        let xdirection = -20.;
        let direction = Vec4d::vector(xdirection, ydirection, zdirection).normalize();
        return Ray::new(origin, direction);
    }
}
