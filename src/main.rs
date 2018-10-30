mod triangle;
mod vec4d;
mod ray;

use triangle::Triangle;
use vec4d::Vec4d;
use ray::Ray;

fn main() {
    let point = Vec4d::point(1., 0., 0.);
    let vec = Vec4d::vector(1., 0., 0.);
    let tri = Triangle::new(point, point + vec, point - vec);
    let ray = Ray::new(point, vec);
    println!("Hello, world!");
    println!("{:?}", tri);
    println!("{:?}", ray);
}
