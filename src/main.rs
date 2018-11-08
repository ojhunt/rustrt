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
            if false {
                for i in 0..(triangles.len()) {
                    let new_object = Box::new(BasicObject::new(&triangles[i..(i + 1)]));
                    scn.add_object(new_object);
                }
            } else {
                let new_object = Box::new(BasicObject::new(&triangles));
                scn.add_object(new_object);
            }
        }
    }
    scn.finalize();
    return scn;
}

fn main() {
    let scn = load_model("models/CornellBox-Empty-CO.obj");
    let mut output = image::GrayImage::new(700, 700);
    let width = output.width() as f64;
    let height = output.height() as f64;
    for (x, y, _pixel) in output.enumerate_pixels_mut() {
        let xp = (x as f64 - width / 2.) / (width / 2.);
        let yp = -(y as f64 - height / 2.) / (height / 2.);
        let ray : Ray;
        if true {
            let origin = Vec4d::point(0., 1., 3.);
            let xdirection = 10. * xp;
            let ydirection = 10. * yp;
            let zdirection = -20.;
            let direction = Vec4d::vector(xdirection, ydirection, zdirection).normalize();
            ray = Ray::new(origin, direction);
        } else {
            let origin = Vec4d::point(20. * xp, 20. * yp, -10.);
            ray = Ray::new(origin, Vec4d::vector(0., 0., 1.));
        }
        match scn.intersect(ray) {
            None => continue,
            Some(Collision{distance:d, uv:_}) => *_pixel = image::Luma([255 - (d * 30.) as u8])
        }
    }
    output.save("image.png").unwrap();
    println!("Done!");
}
