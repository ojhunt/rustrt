use casefopen;
use colour::Colour;
use fragment::Fragment;
use genmesh::*;
use image;
use image::*;
use material;
use material::MaterialCollisionInfo;
use material::{Material, Transparency};
use obj::{IndexTuple, Obj};
use objects::Mesh;
use ray::Ray;
use ray::RayContext;
use scene::MaterialIdx;
use scene::NormalIdx;
use scene::Scene;
use scene::TextureIdx;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use texture::{Texture, TextureCoordinateIdx};
use triangle::Triangle;
use vectors::Vec2d;
use vectors::Vec4d;

trait RawSurfaceValue: Clone + Clone + Copy {
    fn empty() -> Self;
}

impl RawSurfaceValue for Colour {
    fn empty() -> Self {
        return Colour::RGB(0.0, 0.0, 0.0);
    }
}

trait TextureSurfaceValue<Raw: RawSurfaceValue> {
    fn raw_for_fragment(&self, s: &Scene, f: &Fragment) -> Raw;
    fn gradient(&self, s: &Scene, point: Vec2d) -> (f64, f64);
}

impl TextureSurfaceValue<Colour> for TextureIdx {
    fn raw_for_fragment(&self, s: &Scene, f: &Fragment) -> Colour {
        return Colour::from(s.get_texture(*self).sample(f.uv));
    }
    fn gradient(&self, s: &Scene, uv: Vec2d) -> (f64, f64) {
        return s.get_texture(*self).gradient(uv);
    }
}

#[derive(Debug, Copy, Clone)]
enum WFSurfaceProperty<Raw: Copy + RawSurfaceValue, Texture: Copy + TextureSurfaceValue<Raw>> {
    None,
    Single(Raw),
    Texture(Texture),
    Complex(Raw, Texture),
}

trait MergeValues {
    fn merge(&self, other: Self) -> Self;
}

impl MergeValues for Colour {
    fn merge(&self, other: Self) -> Self {
        match (self, other) {
            (Colour::RGB(r1, g1, b1), Colour::RGB(r2, g2, b2)) => {
                Colour::RGB(r1 * r2, b1 * b2, g1 * g2)
            }
        }
    }
}

impl<Raw: Copy + RawSurfaceValue + MergeValues, Texture: Copy + TextureSurfaceValue<Raw>>
    WFSurfaceProperty<Raw, Texture>
{
    pub fn new() -> WFSurfaceProperty<Raw, Texture> {
        WFSurfaceProperty::None
    }
    pub fn raw_for_fragment(&self, scene: &Scene, fragment: &Fragment) -> Raw {
        return match self {
            WFSurfaceProperty::None => Raw::empty(),
            WFSurfaceProperty::Single(v) => *v,
            WFSurfaceProperty::Texture(t) => t.raw_for_fragment(scene, fragment),
            WFSurfaceProperty::Complex(c, t) => c.merge(t.raw_for_fragment(scene, fragment)),
            _ => panic!(),
        };
    }
    pub fn option_for_fragment(&self, scene: &Scene, fragment: &Fragment) -> Option<Raw> {
        return match self {
            WFSurfaceProperty::None => None,
            raw => Some(self.raw_for_fragment(scene, fragment)),
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WFMaterial {
    // From http://paulbourke.net/dataformats/mtl/
    ambient_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ka
    diffuse_colour: WFSurfaceProperty<Colour, TextureIdx>, // Kd
    specular_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ks
    bump_map: Option<TextureIdx>,
    emissive_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ke
    transparent_colour: Option<Colour>,                     // Tf
    transparency: Transparency, // d -- seriously who came up with these names?
    specular_exponent: Option<f64>, // Ns
    sharpness: Option<f64>,
    index_of_refraction: Option<f64>, // Ni
    illumination_model: usize,
}

fn perturb_normal(bump: Option<TextureIdx>, f: &Fragment, s: &Scene) -> Vec4d {
    if bump.is_none() {
        return f.normal;
    }
    let map = s.get_texture(bump.unwrap());
    let (fu, fv) = {
        let (u, v) = map.gradient(f.uv);
        (u * 0.2, v * 0.2)
    };
    let normal = f.normal;
    let ndpdv = normal.cross(f.dpdv);
    let ndpdu = normal.cross(f.dpdu);
    let mut perturbed_normal = normal + (fu * ndpdv - fv * ndpdu);
    if perturbed_normal.dot(perturbed_normal) == 0.0 {
        perturbed_normal = normal;
    }
    // new_info.ambient_colour = Colour::from(
    //     (n + fu * ndpdv - fv * ndpdu).normalize() * 0.5 + Vec4d::vector(0.5, 0.5, 0.5),
    // );
    // new_info.diffuse_colour = Colour::from(temp);
    // new_info.diffuse_colour = new_info.ambient_colour; //Colour::RGB(0.5, f.uv.0.fract(), f.uv.1.fract());
    return perturbed_normal.normalize();
}

impl Material for WFMaterial {
    fn is_light(&self) -> bool {
        match self.emissive_colour {
            WFSurfaceProperty::None => false,
            _ => true,
        }
    }
    fn compute_surface_properties(
        &self,
        s: &Scene,
        ray: &Ray,
        f: &Fragment,
    ) -> MaterialCollisionInfo {
        let normal = perturb_normal(self.bump_map, f, s);
        let mut result = MaterialCollisionInfo {
            ambient_colour: self.ambient_colour.raw_for_fragment(s, f),
            diffuse_colour: self.diffuse_colour.raw_for_fragment(s, f),
            specular_colour: self.specular_colour.raw_for_fragment(s, f),
            emissive_colour: self.emissive_colour.option_for_fragment(s, f),
            normal: normal,
            position: f.position,
            transparent_colour: None,
            secondaries: vec![],
        };
        if self.illumination_model < 5 {
            return result;
        }

        // Basic reflection
        let reflected_ray = (-2.0 * f.view.dot(normal) * normal + f.view).normalize();

        if self.illumination_model == 5 {
            result.secondaries.push((
                Ray::new(
                    f.position + reflected_ray * 0.01,
                    reflected_ray,
                    Some(ray.ray_context.clone()),
                ),
                result.specular_colour,
                1.0,
            ));
            return result;
        }

        let mut transparent_colour = if let Some(transparent_colour) = result.transparent_colour {
            Colour::RGB(1.0, 1.0, 1.0)
        } else {
            Colour::RGB(1.0, 1.0, 1.0) //return result;
        };

        let (refracted_vector, new_context): (Vec4d, RayContext) = match self.index_of_refraction {
            None => (f.view, ray.ray_context.clone()),
            Some(ior) => {
                let V = f.view * -1.0;
                let in_object = V.dot(f.true_normal) > 0.0;
                let (ni, nt, new_context) = if in_object {
                    let new_context = ray.ray_context.exit_material();
                    (
                        ray.ray_context.current_ior_or(ior),
                        new_context.current_ior_or(1.0),
                        new_context,
                    )
                } else {
                    // result.diffuse_colour = Colour::RGB(0.0, 0.0, 100.0);
                    // result.ambient_colour = Colour::RGB(0.0, 0.0, 100.0);
                    // return result;
                    let new_context = ray.ray_context.enter_material(ior);
                    (ray.ray_context.current_ior_or(1.0), ior, new_context)
                };
                let mut nr = ni / nt;
                // nr = 1.0 / nr;

                let n_dot_v = normal.dot(V);

                let inner = 1.0 - nr * nr * (1.0 - n_dot_v * n_dot_v);
                if inner < 0.0 {
                    // Total internal reflection
                    // return result;
                    (reflected_ray, ray.ray_context.clone())
                } else {
                    (
                        ((nr * n_dot_v - inner.sqrt()) * normal - nr * V).normalize(),
                        new_context,
                    )
                }
            }
        };
        result.secondaries.push((
            Ray::new(
                f.position + refracted_vector * 0.01,
                refracted_vector,
                Some(new_context),
            ),
            transparent_colour,
            1.0,
        ));

        return result;
    }
}

fn colour_from_slice(colour: Option<[f32; 3]>) -> Option<Colour> {
    match colour {
        None => None,
        Some([r, g, b]) => Some(Colour::RGB(r as f64, g as f64, b as f64)),
    }
}

fn load_bumpmap<F: FnMut(&str, bool) -> Option<TextureIdx>>(
    texture: &Option<String>,
    mut texture_loader: F,
) -> (Option<TextureIdx>, F) {
    return match texture {
        None => (None, texture_loader),
        Some(texture_name) => (texture_loader(texture_name, true), texture_loader),
    };
}

fn load_surface_colour<F: FnMut(&str, bool) -> Option<TextureIdx>>(
    colour: Option<[f32; 3]>,
    texture: &Option<String>,
    mut texture_loader: F,
) -> (WFSurfaceProperty<Colour, TextureIdx>, F) {
    let real_colour = match colour {
        None => None,
        Some([r, g, b]) => Some(Colour::RGB(r as f64, g as f64, b as f64)),
    };
    let real_texture = match texture {
        None => None,
        Some(texture_name) => texture_loader(texture_name, false),
    };

    return (
        match (real_colour, real_texture) {
            (Some(Colour::RGB(r, g, b)), Some(texture)) if r == 0.0 && g == 0.0 && b == 0.0 => {
                WFSurfaceProperty::Texture(texture)
            }
            (Some(colour), Some(texture)) => WFSurfaceProperty::Complex(colour, texture),
            (Some(Colour::RGB(r, g, b)), None) if r != 0.0 && g != 0.0 && b != 0.0 => {
                WFSurfaceProperty::Single(Colour::RGB(r, g, b))
            }
            (None, Some(texture)) => WFSurfaceProperty::Texture(texture),
            _ => WFSurfaceProperty::None,
        },
        texture_loader,
    );
}

impl WFMaterial {
    pub fn new<F: FnMut(&str, bool) -> Option<TextureIdx>>(
        mat: &obj::Material,
        texture_loader: F,
    ) -> WFMaterial {
        let opt_f32_to_f64 = |o: Option<f32>| {
            if let Some(v) = o {
                Some(v as f64)
            } else {
                None
            }
        };

        let (ambient, f) = load_surface_colour(mat.ka, &mat.map_ka, texture_loader);
        let (diffuse, f1) = load_surface_colour(mat.kd, &mat.map_kd, f);
        let (specular, f2) = load_surface_colour(mat.ks, &mat.map_ks, f1);
        let (emission, f3) = load_surface_colour(mat.ke, &mat.map_ke, f2);
        let (bump_map, _) = load_bumpmap(&mat.map_bump, f3);

        WFMaterial {
            ambient_colour: ambient,
            diffuse_colour: diffuse,
            specular_colour: specular,
            emissive_colour: emission,
            bump_map: bump_map,
            transparent_colour: colour_from_slice(mat.tf),
            specular_exponent: opt_f32_to_f64(mat.ns),
            index_of_refraction: opt_f32_to_f64(mat.ni),
            transparency: if let Some(d) = mat.d {
                Transparency::Constant(d as f64)
            } else {
                Transparency::Opaque
            },
            sharpness: Some(1.),
            illumination_model: mat.illum.unwrap_or(4) as usize,
        }
    }
}

fn vecf32_to_point(v: [f32; 3]) -> Vec4d {
    Vec4d::point(v[0] as f64, v[1] as f64, v[2] as f64)
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
