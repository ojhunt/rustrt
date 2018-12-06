use camera::Camera;
use casefopen;
use collision::Collision;
use compound_object::CompoundObject;
use genmesh::*;
use image::*;
use intersectable::Intersectable;
use material;
use obj::{IndexTuple, Obj};
use objects::Mesh;
use rand::{thread_rng, Rng};
use ray::Ray;
use shader::Shadable;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use texture::{Texture, TextureCoordinateIdx};
use triangle::Triangle;
use vectors::{Vec2d, Vec4d};
use wavefront_material::*;

fn vecf32_to_point(v: [f32; 3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
}

#[derive(Debug, Copy, Clone)]
pub struct MaterialIdx(usize);

#[derive(Debug, Copy, Clone)]
pub struct TextureIdx(usize);

#[derive(Debug, Copy, Clone)]
pub struct NormalIdx(pub usize);

impl NormalIdx {
    pub fn get(&self, s: &Scene) -> Vec4d {
        let NormalIdx(idx) = *self;
        return s.get_normal(idx);
    }
}

#[derive(Debug)]
pub struct Scene {
    path: PathBuf,
    directory: PathBuf,
    normals: Vec<Vec4d>,
    positions: Vec<Vec4d>,
    materials: Vec<Box<material::Material>>,
    texture_coords: Vec<Vec2d>,
    textures: Vec<Texture>,
    _scene: CompoundObject,
}

impl Scene {
    pub fn new(path: &str) -> Scene {
        let real_path = Path::new(path).canonicalize().unwrap();
        Scene {
            path: real_path.clone(),
            directory: real_path.parent().unwrap().to_owned(),
            normals: Vec::new(),
            positions: Vec::new(),
            materials: Vec::new(),
            texture_coords: Vec::new(),
            textures: Vec::new(),
            _scene: CompoundObject::new(),
        }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self._scene.add_object(object)
    }

    pub fn intersect<'a>(&'a self, ray: &Ray) -> Option<(Collision, &'a Shadable)> {
        return self._scene.intersect(ray, ray.min, ray.max);
    }

    pub fn finalize(&mut self) {
        self._scene.finalize();
    }

    pub fn get_normal(&self, idx: usize) -> Vec4d {
        let n = self.normals[idx];
        assert!(n.w == 0.0);
        return n;
    }

    pub fn get_texture_coordinate(&self, idx: usize) -> Vec2d {
        let n = self.texture_coords[idx];
        return n;
    }

    pub fn get_material(&self, MaterialIdx(idx): MaterialIdx) -> &material::Material {
        return &*self.materials[idx];
    }

    pub fn get_texture(&self, TextureIdx(idx): TextureIdx) -> &Texture {
        return &self.textures[idx];
    }

    pub fn render<C: Camera>(&self, camera: &C, size: usize) -> DynamicImage {
        let mut result = image::RgbImage::new(size as u32, size as u32);
        let mut buffer: Vec<Vec<Vec4d>> = vec![];
        for _ in 0..(size * size) {
            buffer.push(vec![]);
        }
        let rays = camera.get_rays(size, size);
        let light_objects = &self._scene.get_lights(self);
        let light_areas: &Vec<f64> = &light_objects.iter().map(|l| l.get_area()).collect();
        let total_area = {
            let mut area = 0.0;
            for light_area in light_areas {
                area += light_area;
            }
            area
        };
        println!("Light count: {}", light_objects.len());

        let lights = if light_objects.len() != 0 {
            let max_lights = 1000;
            let mut remaining_lights = max_lights;
            let mut lights: Vec<Vec4d> = vec![];
            for i in 0..light_areas.len() {
                let light_area = light_areas[i];
                let light_count = if i < light_areas.len() - 1 {
                    (max_lights as f64 * (light_area / total_area)) as usize
                } else {
                    remaining_lights
                };
                remaining_lights -= light_count;
                let mut samples = light_objects[i]
                    .get_samples(light_count, self)
                    .iter()
                    .map(|l| l.position)
                    .collect();
                lights.append(&mut samples);
            }
            lights
        } else {
            vec![
                Vec4d::point(2., 3., 0.),
                Vec4d::point(-10., -12., -4.),
                Vec4d::point(-16., 9.5, 4.),
                Vec4d::point(-14., 19.5, -2.),
            ]
        };
        println!("virtual count: {}", lights.len());
        for (x, y, w, ray) in &rays {
            match self.intersect(ray) {
                None => continue,
                Some((c, shadable)) => {
                    let fragment = shadable.compute_fragment(self, ray, &c);
                    let material = match fragment.material {
                        Some(inner) => self.get_material(inner),
                        None => continue,
                    };
                    let surface = material.compute_surface_properties(self, &fragment);
                    let ambient_colour = Vec4d::from(surface.ambient_colour);
                    let diffuse_colour = Vec4d::from(surface.diffuse_colour);

                    let mut colour = ambient_colour * 0.2;
                    if true {
                        if diffuse_colour.length() == 0.0 {
                            continue;
                        }
                        let light_samples = 20;
                        let mut has_intersected = false;
                        for i in 0..light_samples {
                            let light = &lights[thread_rng().gen_range(0, lights.len())];
                            let mut ldir = *light - surface.position;
                            let ldir_len = ldir.dot(ldir).sqrt();
                            ldir = ldir.normalize();
                            if i * 4 < light_samples || has_intersected {
                                let shadow_test = Ray::new_bound(
                                    surface.position,
                                    ldir,
                                    0.01 * ldir_len,
                                    ldir_len * 0.999,
                                );

                                if self.intersect(&shadow_test).is_some() {
                                    has_intersected = true;
                                    continue;
                                }
                            }
                            let diffuse_intensity = ldir.dot(surface.normal) / light_samples as f64;
                            if diffuse_intensity <= 0.0 {
                                continue;
                            }

                            colour = colour + diffuse_colour * diffuse_intensity;
                        }
                    } else {
                        colour = diffuse_colour;
                        // Vec4d::vector(diffuse_colour.x, 1. - c.distance.log10() / 2., 0.0); // ambient_colour + diffuse_colour;
                    }
                    buffer[x + y * size].push(colour * *w);
                }
            }
        }

        for (x, y, _pixel) in result.enumerate_pixels_mut() {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            for v in &buffer[x as usize + y as usize * size] {
                r += v.x;
                g += v.y;
                b += v.z;
            }
            *_pixel = image::Rgb([
                (r * 255.).max(0.).min(255.) as u8,
                (g * 255.).max(0.).min(255.) as u8,
                (b * 255.).max(0.).min(255.) as u8,
            ]);
        }

        return ImageRgb8(result);
    }
}

pub fn load_scene(path: &str) -> Scene {
    let mut scn = Scene::new(path);

    let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(path)).unwrap();

    obj.load_mtls().unwrap();

    let mut texture_map: HashMap<PathBuf, TextureIdx> = HashMap::new();
    let mut textures: Vec<Texture> = Vec::new();
    let directory = scn.directory.clone();
    let mut load_texture = |textures: &mut Vec<Texture>, file: &str, need_bumpmap: bool| {
        let resolved_path = directory.join(file);
        if let Some(result) = texture_map.get(&resolved_path) {
            if need_bumpmap {
                let TextureIdx(idx) = result;
                textures[*idx].generate_derivate_maps();
            }
            return Some(*result);
        }

        let format = if let Some(extension) = resolved_path.extension() {
            match (extension.to_str().unwrap()).to_lowercase().as_str() {
                "png" => ImageFormat::PNG,
                "pnm" => ImageFormat::PNG,
                "jpeg" => ImageFormat::JPEG,
                "jpg" => ImageFormat::JPEG,
                x => panic!("Extension {}", x),
            }
        } else {
            panic!();
        };

        let image = match casefopen::open(&resolved_path) {
            Ok(file) => {
                let mut buffer = std::io::BufReader::new(file);
                match image::load(buffer, format) {
                    Ok(image) => image,
                    Err(msg) => panic!("Failed to open {:?} with error: {}", resolved_path, msg),
                }
            }
            Err(msg) => panic!("Fopen({:?}) failed with {}", resolved_path, msg),
        };
        let texture = Texture::new(resolved_path.to_str().unwrap(), &image);

        let texture_idx = TextureIdx(textures.len());

        textures.push(texture);
        if need_bumpmap {
            let TextureIdx(idx) = texture_idx;
            textures[idx].generate_derivate_maps();
        }
        texture_map.insert(resolved_path, texture_idx);

        return Some(texture_idx);
    };

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
        scn.texture_coords.push(Vec2d(*u as f64, *v as f64));
    }
    let max_tex: usize = scn.texture_coords.len();
    let mut material_map: HashMap<String, (MaterialIdx, bool)> = HashMap::new();
    let mut materials: Vec<Box<material::Material>> = Vec::new();

    let object_count = obj.objects.len();
    for object_index in 0..object_count {
        let mut index_for_material = |mat: &obj::Material| -> (MaterialIdx, bool) {
            let name = &mat.name;
            if let Some(existing) = material_map.get(name) {
                return *existing;
            }
            let material: Box<material::Material> =
                Box::new(WFMaterial::new(mat, |file, need_bumpmap| {
                    load_texture(&mut textures, file, need_bumpmap)
                }));
            let is_light = material.is_light();
            materials.push(material);

            material_map.insert(name.clone(), (MaterialIdx(materials.len() - 1), is_light));
            return (MaterialIdx(materials.len() - 1), is_light);
        };

        let object = &obj.objects[object_index];
        let mut object_triangles: Vec<Triangle> = vec![];

        let group_count = object.groups.len();
        let mut lights: Vec<(usize, usize)> = vec![];
        for group_index in 0..group_count {
            let ref group = &object.groups[group_index];
            let mut is_light = false;
            let (material_index, is_light) = if let Some(ref mat) = group.material {
                let material: &obj::Material = &**mat;
                let (mat, is_light) = index_for_material(material);
                (Some(mat), is_light)
            } else {
                (None, false)
            };
            let mut triangles: Vec<Triangle> = group
                .polys
                .iter()
                .map(|x| *x)
                .vertex(|IndexTuple(p, t, n)| {
                    let n_idx: Option<NormalIdx> = match n {
                        Some(idx) => {
                            let normal = scn.get_normal(idx);
                            if normal.dot(normal) != 0.0 {
                                Some(NormalIdx(idx))
                            } else {
                                None
                            }
                        }
                        None => None,
                    };
                    let t_idx: Option<TextureCoordinateIdx> = match t {
                        Some(idx) => {
                            assert!(idx < max_tex);
                            Some(TextureCoordinateIdx(idx))
                        }
                        None => None,
                    };
                    (vecf32_to_point(obj.position[p]), t_idx, n_idx)
                })
                .triangulate()
                .map(|genmesh::Triangle { x, y, z }| {
                    if let Some(nidx) = x.2 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    if let Some(nidx) = y.2 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    if let Some(nidx) = z.2 {
                        let n = nidx.get(&scn);
                        assert!(n.dot(n) != 0.0);
                    };
                    assert!(material_index.is_some());
                    Triangle::new(material_index, x, y, z)
                })
                .collect();
            object_triangles.append(&mut triangles);
        }

        let new_object = Box::new(Mesh::new(&object_triangles));
        scn.add_object(new_object);
    }
    scn.materials.append(&mut materials);
    scn.textures.append(&mut textures);

    scn.finalize();
    return scn;
}
