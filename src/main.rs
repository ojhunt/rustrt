extern crate image;

mod triangle;
mod vec4d;
mod ray;
mod scene;

use triangle::Triangle;
use vec4d::Vec4d;
use ray::Ray;
use scene::Scene;


fn main() {
    let tri = Triangle::new(Vec4d::point(-10., -10., 10.), Vec4d::point(0., 10., 10.), Vec4d::point(10., -10., 10.));

    let mut scn = Scene::new();
    scn.addTriangle(&tri);
    println!("Hello, world!");
    println!("{:?}", scn);

    let mut output = image::GrayImage::new(100,100);
    let width = output.width() as f64;
    let height = output.height() as f64;
    for (x, y, pixel) in output.enumerate_pixels_mut() {
        let origin = Vec4d::point(0., 0., -10.);
        let xdirection = (45. * (x as f64 - width / 2.) / (width / 2.)).to_radians().sin();
        let ydirection = (-45. * (y as f64 - height / 2.) / (height / 2.)).to_radians().cos();
        let zdirection = (1. - xdirection * xdirection).sqrt();
        let direction = Vec4d::vector(xdirection, ydirection, zdirection).normalize();
        let ray = Ray::new(origin, direction);
        let triangle = scn.intersect(ray);
    }
}
