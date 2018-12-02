use camera::Camera;
use casefopen;
use collision::Collision;
use colour::Colour;
use compound_object::CompoundObject;
use genmesh::*;
use image::*;
use intersectable::Intersectable;
use material;
use obj::{IndexTuple, Obj};
use objects::Mesh;
use ray::Ray;
use shader::Shadable;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use texture::{Texture, TextureCoordinateIdx};
use triangle::Triangle;
use vectors::{Vec2d, Vec4d};
use wavefront_material::WFMaterial;

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
        let mut buffer = vec![(0 as f64, 0 as f64, 0 as f64); size * size];

        let rays = camera.get_rays(size, size);
        let lights = [Vec4d::point(8.5, -12.0, 12.0), Vec4d::point(2., 3., 0.)]; //, Vec4d::point(-10., -12., -4.)];
        for x in 0..size {
            for y in 0..size {
                let ray = &rays[x + size * y];
                match self.intersect(ray) {
                    None => continue,
                    Some((c, shadable)) => {
                        let fragment = shadable.compute_fragment(self, ray, c);
                        let material = match fragment.material {
                            Some(inner) => self.get_material(inner),
                            None => continue,
                        };
                        let surface = material.compute_surface_properties(self, &fragment);
                        let ambient_colour = Vec4d::from(surface.ambient_colour);
                        let diffuse_colour = Vec4d::from(surface.diffuse_colour);

                        let mut colour = ambient_colour * 0.2;
                        if true {
                            for light in lights.iter() {
                                let mut ldir = *light - surface.position;
                                let ldir_len = ldir.dot(ldir).sqrt();
                                ldir = ldir.normalize();

                                let shadow_test = Ray::new_bound(
                                    surface.position,
                                    ldir,
                                    0.001 * ldir_len,
                                    ldir_len * 0.999,
                                );

                                if true && self.intersect(&shadow_test).is_some() {
                                    continue;
                                }

                                let diffuse_intensity =
                                    ldir.dot(surface.normal) / lights.len() as f64;
                                if diffuse_intensity <= 0.0 {
                                    continue;
                                }

                                colour = colour + diffuse_colour * diffuse_intensity;
                            }
                        } else {
                            colour = diffuse_colour;
                        }
                        buffer[x + y * size] = (colour.x, colour.y, colour.z);
                    }
                }
            }
        }

        for (x, y, _pixel) in result.enumerate_pixels_mut() {
            let (r, g, b) = buffer[x as usize + y as usize * size];
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
    let mut load_texture = |textures: &mut Vec<Texture>, file: &str| {
        let resolved_path = directory.join(file);
        if let Some(result) = texture_map.get(&resolved_path) {
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
    let mut material_map: HashMap<String, MaterialIdx> = HashMap::new();
    let mut materials: Vec<Box<material::Material>> = Vec::new();

    let object_count = obj.objects.len();
    for object_index in 0..object_count {
        let mut index_for_material = |mat: &obj::Material| -> MaterialIdx {
            let name = &mat.name;
            if let Some(existing) = material_map.get(name) {
                return *existing;
            }
            materials.push(Box::new(WFMaterial::new(mat, |file| {
                load_texture(&mut textures, file)
            })));
            material_map.insert(name.clone(), MaterialIdx(materials.len() - 1));
            return MaterialIdx(materials.len() - 1);
        };

        let object = &obj.objects[object_index];
        let mut object_triangles: Vec<Triangle> = vec![];

        let group_count = object.groups.len();

        for group_index in 0..group_count {
            let ref group = &object.groups[group_index];
            let material_index = if let Some(ref mat) = group.material {
                let material: &obj::Material = &**mat;
                Some(index_for_material(material))
            } else {
                None
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
