extern crate image;
extern crate genmesh;
extern crate itertools;


mod basic_object;
mod bounding_box;
mod bvh;
mod camera;
mod collision;
mod compound_object;
mod intersectable;
mod objects;
mod ray;
mod scene;
mod triangle;
mod vec4d;

use genmesh::{*};
use triangle::Triangle;
use vec4d::Vec4d;
use ray::Ray;
use scene::Scene;
use collision::Collision;
use bounding_box::*;
use std::time::{Instant,Duration};

extern crate obj;

use obj::{*};
use objects::{*};

use std::path::Path;
fn vecf32_to_point(v: [f32;3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
}
fn vecf32_to_vector(v: [f32;3]) -> Vec4d {
    Vec4d::vector(v[0] as f64, v[1] as f64, v[2] as f64)
}

fn load_model(path: &str) -> Scene {
    let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(path)).unwrap();
    obj.load_mtls().unwrap();

    let mut scn = Scene::new();
    let mut bounds = BoundingBox::new();
    for o in &obj.objects {
        for g in &o.groups {
            let triangles: Vec<Triangle> = g.polys
                .iter()
                .map(|x| *x)
                .vertex(|IndexTuple(p, t, n)| {
                    (vecf32_to_point(obj.position[p]),
                    t.map_or([0., 0.], |t| obj.texture[t]), 
                    vecf32_to_vector(n.map_or([1., 0., 0.], |n| obj.normal[n] )))
                })
                .triangulate()
                .map(|genmesh::Triangle{x,y,z}| Triangle::new(x.0,y.0,z.0))
                .collect();
            if true {
                let step_size = 1;
                for i in 0..(triangles.len() / step_size) {
                    let new_object = Box::new(BasicObject::new(&triangles[i*step_size..(i + 1)*step_size]));
                    bounds = bounds.merge_with_bbox((*new_object).bounds());
                    scn.add_object(new_object);
                }
            } else {
                let new_object = Box::new(BasicObject::new(&triangles));
                scn.add_object(new_object);
            }
        }
    }
    println!("Bounds: {:?}", bounds);
    scn.finalize();
    return scn;
}

fn main() {
    let scn = load_model("models/sponza.obj");
    let mut output = image::GrayImage::new(700, 700);
    let width = output.width() as f64;
    let height = output.height() as f64;
    let mut buffer : [[f64; 700]; 700] = [[std::f64::INFINITY; 700]; 700];
    let mut minimum = std::f64::INFINITY;
    let mut maximum = -std::f64::INFINITY;
    let start = std::time::Instant::now();
    for x in 0..700 {
        for y in 0..700 {
            let xp = (x as f64 - width / 2.) / (width / 2.);
            let yp = -(y as f64 - height / 2.) / (height / 2.);
            let ray : Ray;
            if true {
                let origin = Vec4d::point(10., 2., 0.);
                let zdirection = 10. * xp;
                let ydirection = 10. * yp;
                let xdirection = -20.;
                let direction = Vec4d::vector(xdirection, ydirection, zdirection).normalize();
                ray = Ray::new(origin, direction);
            } else {
                let origin = Vec4d::point(20. * xp, 20. * yp, -10.);
                ray = Ray::new(origin, Vec4d::vector(0., 0., 1.));
            }
            match scn.intersect(ray) {
                None => continue,
                Some(Collision{distance:d, uv:_}) => {
                    if d < minimum {
                        minimum = d;
                    }
                    if d > maximum {
                        maximum = d;
                    }
                    buffer[x][y] = d;
                }
            }
        }
    }
    let end = std::time::Instant::now();
    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Time taken: {}", time);
    let range = maximum - minimum;
    for (x, y, _pixel) in output.enumerate_pixels_mut() {
        let mut d = buffer[x as usize][y as usize];
        d -= minimum;
        d /= range;
        *_pixel = image::Luma([(255. * (1. - d)).max(0.).min(255.) as u8])
    }
    output.save("image.png").unwrap();
    println!("Done!");
}
