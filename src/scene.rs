use camera::Camera;
use collision::Collision;
use compound_object::CompoundObject;
use image::*;
use intersectable::Intersectable;
use ray::Ray;
use shader::Shadable;
use vec4d::Vec4d;

use genmesh::*;
use obj::*;
use objects::*;
use std::path::Path;
use triangle::{NormalIdx, Triangle};

fn vecf32_to_point(v: [f32; 3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
}

#[derive(Debug)]
pub struct Scene {
    normals: Vec<Vec4d>,
    positions: Vec<Vec4d>,
    texture_coords: Vec<(f64, f64)>,
    textures: Vec<String>,
    _scene: CompoundObject,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            normals: Vec::new(),
            positions: Vec::new(),
            texture_coords: Vec::new(),
            textures: Vec::new(),
            _scene: CompoundObject::new(),
        }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self._scene.add_object(object)
    }

    pub fn intersect<'a>(&'a self, ray: Ray) -> Option<(Collision, &'a Shadable)> {
        return self._scene.intersect(ray, 0.0, std::f64::INFINITY);
    }

    pub fn finalize(&mut self) {
        self._scene.finalize();
    }

    pub fn get_normal(&self, idx: usize) -> Vec4d {
        let n = self.normals[idx];
        assert!(n.w == 0.0);
        return n;
    }

    pub fn render(&self, camera: &Camera, size: usize) -> DynamicImage {
        let mut result = image::RgbImage::new(size as u32, size as u32);
        let mut buffer = vec![(0 as f64, 0 as f64, 0 as f64); size * size];

        let mut min_depth = std::f64::INFINITY;
        let mut max_depth = -std::f64::INFINITY;
        let mut min_nodecount = 0;
        let mut max_nodecount = 0;
        let mut min_intersectount = 0;
        let mut max_intersectcount = 0;
        let rays = camera.get_rays(size, size);
        for x in 0..size {
            for y in 0..size {
                let ray = rays[x + size * y];
                match self.intersect(ray) {
                    None => continue,
                    Some((c, shadable)) => {
                        max_depth = max_depth.max(c.distance);
                        min_depth = min_depth.min(c.distance);
                        max_nodecount = max_nodecount.max(c.node_count);
                        min_nodecount = min_nodecount.min(c.node_count);
                        max_intersectcount = max_intersectcount.max(c.intersection_count);
                        min_intersectount = min_intersectount.min(c.intersection_count);
                        let fragment = shadable.compute_fragment(self, ray, c);
                        let normal = fragment.normal * 0.5 + Vec4d::vector(0.5, 0.5, 0.5);
                        buffer[x + y * size] = (normal.x, normal.y, normal.z);
                    }
                }
            }
        }

        println!(
            "Minimum intersections: {}, max: {}",
            min_intersectount, max_intersectcount
        );
        for (x, y, _pixel) in result.enumerate_pixels_mut() {
            if false {
                let (d, ic, nc) = buffer[x as usize + y as usize * size];

                let scaled_depth = (255. * (1. - (d - min_depth) / (max_depth - min_depth)))
                    .max(0.)
                    .min(255.) as u8;
                let scaled_intersection_count = (255. * (ic - min_intersectount as f64) as f64
                    / (max_intersectcount - min_intersectount) as f64)
                    .max(0.)
                    .min(255.) as u8;
                let scaled_node_count = ((nc - min_nodecount as f64) as f64
                    / (max_nodecount - min_nodecount) as f64)
                    .min(0.)
                    .max(255.) as u8;
                *_pixel = image::Rgb([
                    scaled_depth * 1,
                    scaled_intersection_count * 0,
                    scaled_node_count * 0,
                ]);
            } else {
                let (r, g, b) = buffer[x as usize + y as usize * size];
                *_pixel = image::Rgb([
                    (r * 255.).max(0.).min(255.) as u8,
                    (g * 255.).max(0.).min(255.) as u8,
                    (b * 255.).max(0.).min(255.) as u8,
                ]);
            }
        }

        return ImageRgb8(result);
    }
}

pub fn load_scene(path: &str) -> Scene {
    let mut scn = Scene::new();

    let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(path)).unwrap();
    obj.load_mtls().unwrap();
    scn.textures = obj.material_libs.to_vec();
    for [x, y, z] in obj.position.iter() {
        scn.positions
            .push(Vec4d::point(*x as f64, *y as f64, *z as f64));
    }
    for [x, y, z] in obj.normal.iter() {
        let n = Vec4d::vector(*x as f64, *y as f64, *z as f64);
        if n.dot(n) == 0.0 {
            scn.normals.push(Vec4d::vector(0.0, 0.0, 0.0));
        } else {
            scn.normals.push(n);
        }
    }
    for [u, v] in obj.texture.iter() {
        scn.texture_coords.push((*u as f64, *v as f64));
    }

    let object_count = obj.objects.len();
    for object_index in 0..object_count {
        let object = &obj.objects[object_index];
        let mut object_triangles: Vec<Triangle> = vec![];

        let group_count = object.groups.len();

        for group_index in 0..group_count {
            let mut triangles: Vec<Triangle> = object.groups[group_index]
                .polys
                .iter()
                .map(|x| *x)
                .vertex(|IndexTuple(p, t, n)| {
                    let n_idx = match n {
                        Some(idx) => {
                            let normal = scn.get_normal(idx);
                            if normal.dot(normal) != 0.0 {
                                Some(NormalIdx::NormalIdx(idx))
                            } else {
                                None
                            }
                        }
                        None => None,
                    };
                    (vecf32_to_point(obj.position[p]), n_idx, t)
                })
                .triangulate()
                .map(|genmesh::Triangle { x, y, z }| {
                    if let Some(nidx) = x.1 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    if let Some(nidx) = y.1 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    if let Some(nidx) = z.1 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    Triangle::new(x, y, z)
                })
                .collect();
            object_triangles.append(&mut triangles);
        }

        let new_object = Box::new(Mesh::new(&object_triangles));
        scn.add_object(new_object);
    }
    scn.finalize();
    return scn;
}
