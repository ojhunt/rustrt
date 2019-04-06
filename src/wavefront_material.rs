#[allow(unused_imports)]
use crate::scene::MaterialIdx;
use crate::material::EmissionCoefficients;
use crate::scene::SceneSettings;
use crate::colour::Colour;
use crate::fragment::Fragment;
use genmesh::*;
use crate::material::MaterialCollisionInfo;
use crate::material::{Material, Transparency};
use obj::{IndexTuple, Obj};
use crate::objects::Mesh;
use crate::ray::Ray;
use crate::scene::NormalIdx;
use crate::scene::Scene;
use crate::scene::TextureIdx;
use std::path::Path;
use crate::texture::TextureCoordinateIdx;
use crate::triangle::Triangle;
use crate::sphere::Sphere;
use crate::vectors::*;

trait RawSurfaceValue: Clone + Clone + Copy {
  type RawType;
  fn empty() -> Self;
  fn from_array(array: Option<[f32; 3]>) -> Option<Self>;
  fn max_value(&self) -> f32;
}

impl RawSurfaceValue for Colour {
  type RawType = Self;
  fn empty() -> Self {
    return Colour::RGB(0.0, 0.0, 0.0);
  }
  fn max_value(&self) -> f32 {
    return self.r().max(self.g().max(self.b()));
  }

  fn from_array(array: Option<[f32; 3]>) -> Option<Self> {
    if let Some([r, g, b]) = array {
      if r.max(g.max(b)) == 0.0 {
        return None;
      }
      return Some(Colour::RGB(r, g, b));
    }
    return None;
  }
}

impl RawSurfaceValue for EmissionCoefficients {
  type RawType = Self;
  fn empty() -> Self {
    return EmissionCoefficients {
      ambient: 0.0,
      diffuse: 0.0,
      specular: 0.0,
    };
  }
  fn from_array(array: Option<[f32; 3]>) -> Option<Self> {
    if let Some([d, a, s]) = array {
      if a.max(d.max(s)) == 0.0 {
        return None;
      }
      return Some(EmissionCoefficients {
        ambient: a,
        diffuse: d,
        specular: s,
      });
    }
    return None;
  }
  fn max_value(&self) -> f32 {
    return self.diffuse.max(self.ambient.max(self.specular));
  }
}
/*
impl<T: RawSurfaceValue> RawSurfaceValue for Option<T> {
  type RawType = T;
  fn empty() -> T {
    return T::empty();
  }

  fn from_array([a, d, s]: [f32; 3]) -> Option<RawType> {}
}*/

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

impl TextureSurfaceValue<EmissionCoefficients> for TextureIdx {
  fn raw_for_fragment(&self, s: &Scene, f: &Fragment) -> EmissionCoefficients {
    let colour = s.get_texture(*self).sample(f.uv);
    return EmissionCoefficients {
      ambient: colour.r(),
      diffuse: colour.g(),
      specular: colour.b(),
    };
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
      (Colour::RGB(r1, g1, b1), Colour::RGB(r2, g2, b2)) => Colour::RGB(r1 * r2, b1 * b2, g1 * g2),
    }
  }
}

impl MergeValues for EmissionCoefficients {
  fn merge(&self, other: Self) -> Self {
    Self {
      ambient: self.ambient * other.ambient,
      diffuse: self.diffuse * other.diffuse,
      specular: self.specular * other.specular,
    }
  }
}

impl<Raw: Copy + RawSurfaceValue + MergeValues, Texture: Copy + TextureSurfaceValue<Raw>>
  WFSurfaceProperty<Raw, Texture>
{
  pub fn raw_for_fragment(&self, scene: &Scene, fragment: &Fragment) -> Raw {
    let result: Raw = match self {
      WFSurfaceProperty::None => Raw::empty(),
      WFSurfaceProperty::Single(v) => {
        let r: Raw = *v;
        r
      }
      WFSurfaceProperty::Texture(t) => {
        let r: Raw = t.raw_for_fragment(scene, fragment);
        r
      }
      WFSurfaceProperty::Complex(_, t) => {
        let r: Raw = t.raw_for_fragment(scene, fragment);
        r
      }
    };
    return result;
  }
  pub fn option_for_fragment(&self, scene: &Scene, fragment: &Fragment) -> Option<Raw> {
    return match self {
      WFSurfaceProperty::None => None,
      _raw => Some(self.raw_for_fragment(scene, fragment)),
    };
  }
}

#[derive(Debug, Clone, Copy)]
pub struct WFMaterial {
  // From http://paulbourke.net/dataformats/mtl/
  ambient_colour: WFSurfaceProperty<Colour, TextureIdx>,  // Ka
  diffuse_colour: WFSurfaceProperty<Colour, TextureIdx>,  // Kd
  specular_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ks
  bump_map: Option<TextureIdx>,
  emissive_colour: WFSurfaceProperty<EmissionCoefficients, TextureIdx>, // Ke
  transparent_colour: Option<Colour>,                                   // Tf
  transparency: Transparency,                                           // d -- seriously who came up with these names?
  specular_exponent: Option<f32>,                                       // Ns
  sharpness: Option<f32>,
  index_of_refraction: Option<f32>, // Ni
  illumination_model: usize,
}

fn perturb_normal(bump: Option<TextureIdx>, f: &Fragment, s: &Scene) -> Vector {
  if bump.is_none() {
    return f.normal;
  }
  let map = s.get_texture(bump.unwrap());
  let (fu, fv) = {
    let (u, v) = map.gradient(f.uv);
    (u, v)
  };
  let normal = f.normal;
  let ndpdv = normal.cross(f.dpdv);
  let ndpdu = normal.cross(f.dpdu);
  let mut perturbed_normal = normal + (fu * ndpdv - fv * ndpdu);
  if perturbed_normal.dot(perturbed_normal) == 0.0 {
    perturbed_normal = normal;
  }
  if perturbed_normal.dot(f.view) > 0.0 {
    perturbed_normal = -perturbed_normal;
  }
  // new_info.ambient_colour = Colour::from(
  //     (n + fu * ndpdv - fv * ndpdu).normalize() * 0.5 + Vector::vector(0.5, 0.5, 0.5),
  // );
  // new_info.diffuse_colour = Colour::from(temp);
  // new_info.diffuse_colour = new_info.ambient_colour; //Colour::RGB(0.5, f.uv.0.fract(), f.uv.1.fract());
  return perturbed_normal.normalize();
}

impl Material for WFMaterial {
  fn is_light(&self) -> bool {
    match self.emissive_colour {
      WFSurfaceProperty::None => false,
      WFSurfaceProperty::Single(value) => value.max_value() > 0.0,
      _ => true,
    }
  }

  fn compute_surface_properties(&self, s: &Scene, _: &Ray, f: &Fragment) -> MaterialCollisionInfo {
    let normal = perturb_normal(self.bump_map, f, s);
    let mut result = MaterialCollisionInfo {
      ambient_colour: self.ambient_colour.raw_for_fragment(s, f),
      diffuse_colour: self.diffuse_colour.raw_for_fragment(s, f),
      specular_colour: self.specular_colour.raw_for_fragment(s, f),
      emissive_colour: self.emissive_colour.option_for_fragment(s, f),
      normal: normal,
      position: f.position,
      reflectivity: None,
      transparent_colour: None,
      index_of_refraction: None,
    };

    if self.illumination_model == 5 {
      result.reflectivity = Some((1.0, result.specular_colour));
      return result;
    }
    if let Transparency::Opaque = self.transparency {
      result.index_of_refraction = self.index_of_refraction;
      result.transparent_colour = self.transparent_colour;
    }
    return result;
  }
}

fn colour_from_slice(colour: Option<[f32; 3]>) -> Option<Colour> {
  match colour {
    None => None,
    Some([r, g, b]) => Some(Colour::RGB(r, g, b)),
  }
}

fn load_bumpmap<F: FnMut(&mut Scene, &str, bool) -> Option<TextureIdx>>(
  scene: &mut Scene,
  texture: &Option<String>,
  mut texture_loader: F,
) -> (Option<TextureIdx>, F) {
  return match texture {
    None => (None, texture_loader),
    Some(texture_name) => (texture_loader(scene, texture_name, true), texture_loader),
  };
}

fn load_surface<Raw: Copy + RawSurfaceValue, F: FnMut(&mut Scene, &str, bool) -> Option<TextureIdx>>(
  scene: &mut Scene,
  colour: Option<[f32; 3]>,
  texture: &Option<String>,
  mut texture_loader: F,
) -> (WFSurfaceProperty<Raw, TextureIdx>, F)
where
  TextureIdx: TextureSurfaceValue<Raw>,
{
  let real_colour = Raw::from_array(colour);
  let real_texture = match texture {
    None => None,
    Some(texture_name) => texture_loader(scene, texture_name, false),
  };

  let result = match (real_colour, real_texture) {
    (Some(raw), Some(texture)) if raw.max_value() == 0.0 => WFSurfaceProperty::Texture(texture),
    (Some(colour), Some(texture)) => WFSurfaceProperty::Complex(colour, texture),
    (Some(raw), None) if raw.max_value() != 0.0 => (WFSurfaceProperty::Single(raw)),
    (None, Some(texture)) => (WFSurfaceProperty::Texture(texture)),
    _ => (WFSurfaceProperty::None),
  };
  return (result, texture_loader);
}

impl WFMaterial {
  pub fn new<F: FnMut(&mut Scene, &str, bool) -> Option<TextureIdx>>(
    scene: &mut Scene,
    mat: &obj::Material,
    texture_loader: F,
  ) -> WFMaterial {
    let (ambient, f) = load_surface(scene, mat.ka, &mat.map_ka, texture_loader);
    let (diffuse, f1) = load_surface(scene, mat.kd, &mat.map_kd, f);
    let (specular, f2) = load_surface(scene, mat.ks, &mat.map_ks, f1);
    let (emission, f3) = load_surface(scene, mat.ke, &mat.map_ke, f2);
    let (bump_map, _) = load_bumpmap(scene, &mat.map_bump, f3);

    WFMaterial {
      ambient_colour: ambient,
      diffuse_colour: diffuse,
      specular_colour: specular,
      emissive_colour: emission,
      bump_map: bump_map,
      transparent_colour: colour_from_slice(mat.tf),
      specular_exponent: mat.ns,
      index_of_refraction: mat.ni,
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

fn vecf32_to_point(v: [f32; 3]) -> Point {
  Vector::point(v[0] as f64, v[1] as f64, v[2] as f64)
}

pub fn load_scene(settings: &SceneSettings) -> Scene {
  let mut scn = Scene::new(settings);
  let mut obj = Obj::<Polygon<IndexTuple>>::load(&Path::new(&settings.scene_file)).unwrap();

  obj.load_mtls().unwrap();

  for [x, y, z] in obj.position.iter() {
    scn.positions.push(Vector::point(*x as f64, *y as f64, *z as f64));
  }
  for [x, y, z] in obj.normal.iter() {
    let n = Vector::vector(*x as f64, *y as f64, *z as f64);
    if n.dot(n) == 0.0 {
      scn.normals.push(Vector::vector(0.0, 0.0, 0.0));
    } else {
      scn.normals.push(n);
    }
  }
  for [u, v] in obj.texture.iter() {
    scn.texture_coords.push(Vec2d(*u as f64, *v as f64));
  }
  let max_tex: usize = scn.texture_coords.len();
  let default_material = scn.default_material();
  let object_count = obj.objects.len();
  // let mut index_for_material = |mat: &obj::Material| -> (MaterialIdx, bool) {
  //   let name = &mat.name;
  //   if let Some(existing) = material_map.get(name) {
  //     return *existing;
  //   }
  //   let material: Box<material::Material> = Box::new(WFMaterial::new(mat, |file, need_bumpmap| {
  //     load_texture(&mut textures, file, need_bumpmap)
  //   }));
  //   let is_light = material.is_light();
  //   let index = scn.add_material(material);

  //   material_map.insert(name.clone(), (MaterialIdx(materials.len() - 1), is_light));
  //   return (index, is_light);
  // };

  for object_index in 0..object_count {
    let object = &obj.objects[object_index];
    let mut object_triangles: Vec<Triangle> = vec![];

    let group_count = object.groups.len();

    for group_index in 0..group_count {
      let ref group = &object.groups[group_index];
      let material_index = if let Some(ref mat) = group.material {
        let material: &obj::Material = &**mat;
        let mat = scn.get_or_create_material(&material.name, |scene| {
          return Some(Box::new(WFMaterial::new(scene, mat, |scene, file, need_bumpmap| {
            scene.load_texture(file, need_bumpmap)
          })));
        });
        Some(mat.0)
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

          Triangle::new(
            if let Some(material) = material_index {
              if true || scn.get_material(material).is_light() {
                material
              } else {
                default_material
              }
            } else {
              default_material
            },
            x,
            y,
            z,
          )
        })
        .collect();
      object_triangles.append(&mut triangles);
    }

    let new_object = Box::new(Mesh::new(&object_triangles));
    scn.add_object(new_object);

    let sphere = box Sphere::new(Vector::point(0.0, 1.0, 0.0), 0.399, scn.mirror_material());
    scn.add_object(sphere);
  }
  scn.finalize();
  return scn;
}
