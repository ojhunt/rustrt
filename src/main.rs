extern crate image;
extern crate genmesh;

mod triangle;
mod vec4d;
mod ray;
mod scene;

use genmesh::{*};
use triangle::Triangle;
use vec4d::Vec4d;
use ray::Ray;
use scene::Scene;

extern crate obj;

use obj::{*};

use std::path::Path;
fn vecf32_to_point(v: [f32;3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
}
fn vecf32_to_vector(v: [f32;3]) -> Vec4d {
    Vec4d::vector(v[0] as f64, v[1] as f64, v[2] as f64)
}

fn load_model(path: &str) -> Scene {
    let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(path)).unwrap();
    // let _ = sponza.load_mtls();
    obj.load_mtls().unwrap();
    println!("{:?}", obj.position);

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
            for triangle in triangles {
                scn.add_triangle(&triangle);
            }
        }
    }
    let vs = &obj.position;
    for i in 0..vs.len()/3 {
        let [vx0, vy0, vz0] = vs[i*3 + 0];
        let [vx1, vy1, vz1] = vs[i*3 + 1];
        let [vx2, vy2, vz2] = vs[i*3 + 2];
        let v0 = Vec4d::point(vx0 as f64, vy0 as f64, vz0 as f64);
        let v1 = Vec4d::point(vx1 as f64, vy1 as f64, vz1 as f64);
        let v2 = Vec4d::point(vx2 as f64, vy2 as f64, vz2 as f64);
        scn.add_triangle(&Triangle::new(v0, v1, v2));
    }
    return scn;
}

fn main() {
    let scn = load_model("models/CornellBox-Empty-CO.obj");
    println!("{:?}", scn);

    let mut output = image::GrayImage::new(200,200);
    let width = output.width() as f64;
    let height = output.height() as f64;
    for (x, y, _pixel) in output.enumerate_pixels_mut() {
        let xp = (x as f64 - width / 2.) / (width / 2.);
        let yp = -(y as f64 - height / 2.) / (height / 2.);
        let ray : Ray;
        if true {
            let origin = Vec4d::point(0., 0., 4.);
            let xdirection = 15. * xp;
            let ydirection = 15. * yp;
            let zdirection = -15.;
            let direction = Vec4d::vector(xdirection, ydirection, zdirection).normalize();
            ray = Ray::new(origin, direction);
        } else {
            let origin = Vec4d::point(20. * xp, 20. * yp, -10.);
            ray = Ray::new(origin, Vec4d::vector(0., 0., 1.));
        }
        // println!("{}, {}. {:?}", xdirection, ydirection, ray);
        match scn.intersect(ray) {
            None => continue,
            Some((_, d, _)) => *_pixel = image::Luma([255 - (d * 5.) as u8])
        }
    }
    output.save("image.png").unwrap();
}
