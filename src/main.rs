extern crate genmesh;
extern crate image;
extern crate itertools;

extern crate clap;

mod bounding_box;
mod bvh;
mod camera;
mod collision;
mod compound_object;
mod intersectable;
mod mesh;
mod objects;
mod ray;
mod scene;
mod triangle;
mod vec4d;

use bounding_box::*;
use camera::Camera;
use clap::*;
use collision::Collision;
use genmesh::*;
use ray::Ray;
use scene::Scene;
use triangle::Triangle;
use vec4d::Vec4d;

extern crate obj;

use obj::*;
use objects::*;

use std::path::Path;
fn vecf32_to_point(v: [f32; 3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
}
fn vecf32_to_vector(v: [f32; 3]) -> Vec4d {
    Vec4d::vector(v[0] as f64, v[1] as f64, v[2] as f64)
}

fn load_model(path: &str) -> Scene {
    let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(path)).unwrap();
    obj.load_mtls().unwrap();

    let mut scn = Scene::new();
    let mut bounds = BoundingBox::new();
    for o in &obj.objects {
        for g in &o.groups {
            let triangles: Vec<Triangle> = g
                .polys
                .iter()
                .map(|x| *x)
                .vertex(|IndexTuple(p, t, n)| {
                    (
                        vecf32_to_point(obj.position[p]),
                        t.map_or([0., 0.], |t| obj.texture[t]),
                        vecf32_to_vector(n.map_or([1., 0., 0.], |n| obj.normal[n])),
                    )
                })
                .triangulate()
                .map(|genmesh::Triangle { x, y, z }| Triangle::new(x.0, y.0, z.0))
                .collect();
            if true {
                let step_size = 1;
                for i in 0..(triangles.len() / step_size) {
                    let new_object =
                        Box::new(Mesh::new(&triangles[i * step_size..(i + 1) * step_size]));
                    bounds = bounds.merge_with_bbox((*new_object).bounds());
                    scn.add_object(new_object);
                }
            } else {
                let new_object = Box::new(Mesh::new(&triangles));
                scn.add_object(new_object);
            }
        }
    }
    scn.finalize();
    return scn;
}

type ResultBufferType = (f64, usize);

struct SceneSettings {
    pub output_file: String,
    pub scene_file: String,
}

impl SceneSettings {
    pub fn new() -> SceneSettings {
        return SceneSettings {
            output_file: String::new(),
            scene_file: String::new(),
        };
    }
}

fn load_settings() -> SceneSettings {
    let commandline_yaml = load_yaml!("command_line.yml");
    let matches = App::from_yaml(commandline_yaml).get_matches();
    let output_file = matches.value_of("output").unwrap();
    let scene_file = matches.value_of("scene").unwrap();
    let mut settings = SceneSettings::new();
    settings.output_file = output_file.to_string();
    settings.scene_file = scene_file.to_string();
    return settings;
}

fn main() {
    let settings = load_settings();

    const SIZE: usize = 700;
    let scn = load_model(&settings.scene_file);
    let camera = Camera::new(Vec4d::point(10., 2., 0.), Vec4d::point(0.0, 2.0, 0.0), 40.);
    let output = scn.render(&camera, SIZE);

    output.save(settings.output_file).unwrap();
    println!("Done!");
}
