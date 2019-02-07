use crate::material::EmissionCoefficients;
use std::sync::Arc;
use crate::scene::SceneSettings;
use crate::colour::Colour;
use crate::fragment::Fragment;
use genmesh::*;
use crate::material::MaterialCollisionInfo;
use crate::material::{Material, Transparency};
use obj::{IndexTuple, Obj};
use crate::objects::Mesh;
use crate::ray::Ray;
use crate::ray::RayContext;
use crate::scene::NormalIdx;
use crate::scene::Scene;
use crate::scene::TextureIdx;
use std::path::Path;
use crate::texture::TextureCoordinateIdx;
use crate::triangle::Triangle;
use crate::vectors::*;

trait RawSurfaceValue: Clone + Clone + Copy {
  type RawType;
  fn empty() -> Self;
  fn from_array(array: Option<[f32; 3]>) -> Option<Self>;
  fn max_value(&self) -> f64;
}

impl RawSurfaceValue for Colour {
  type RawType = Self;
  fn empty() -> Self {
    return Colour::RGB(0.0, 0.0, 0.0);
  }
  fn max_value(&self) -> f64 {
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
    if let Some([a, d, s]) = array {
      if a.max(d.max(s)) == 0.0 {
        return None;
      }
      return Some(EmissionCoefficients {
        ambient: a as f64,
        diffuse: d as f64,
        specular: s as f64,
      });
    }
    return None;
  }
  fn max_value(&self) -> f64 {
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
  specular_exponent: Option<f64>,                                       // Ns
  sharpness: Option<f64>,
  index_of_refraction: Option<f64>, // Ni
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

  fn compute_surface_properties(&self, s: &Scene, ray: &Ray, f: &Fragment) -> MaterialCollisionInfo {
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
    let reflected_ray = f.view.reflect(normal);

    if self.illumination_model == 5 {
      result.secondaries.push((
        Ray::new(
          f.position + (reflected_ray * 0.01),
          reflected_ray,
          Some(ray.ray_context.clone()),
        ),
        result.specular_colour,
        1.0,
      ));

      return result;
    }

    let transparent_colour = if let Some(transparent_colour) = result.transparent_colour {
      transparent_colour
    } else {
      Colour::RGB(1.0, 1.0, 1.0) //return result;
    };
    let mut refraction_weight = 1.0;
    let (refracted_vector, new_context): (Vector, RayContext) = match self.index_of_refraction {
      None => (f.view, ray.ray_context.clone()),
      Some(ior) => {
        let view = f.view * -1.0;
        let in_object = view.dot(f.true_normal) > 0.0;
        let (ni, nt, new_context) = if in_object {
          let new_context = ray.ray_context.exit_material();
          (
            ray.ray_context.current_ior_or(ior),
            new_context.current_ior_or(1.0),
            new_context,
          )
        } else {
          let new_context = ray.ray_context.enter_material(ior);
          (ray.ray_context.current_ior_or(1.0), ior, new_context)
        };
        let nr = ni / nt;

        let n_dot_v = normal.dot(view);

        let inner = 1.0 - nr * nr * (1.0 - n_dot_v * n_dot_v);
        if inner < 0.0 {
          (reflected_ray, ray.ray_context.clone())
        } else {
          // Schlick approximation of fresnel term
          let r0 = {
            let r0root = (nt - ni) / (nt + ni);
            r0root * r0root
          };
          let fresnel_weight = {
            let one_minus_cos_theta = 1.0 - n_dot_v;
            let squared = one_minus_cos_theta * one_minus_cos_theta;
            let quintupled = squared * squared * one_minus_cos_theta;
            r0 + (1.0 - r0) * quintupled
          };
          if fresnel_weight > 0.02 {
            result.secondaries.push((
              Ray::new(
                f.position + reflected_ray * 0.01,
                reflected_ray,
                Some(ray.ray_context.clone()),
              ),
              result.specular_colour,
              fresnel_weight,
            ));
            refraction_weight -= fresnel_weight;
          }
          (
            ((nr * n_dot_v - inner.sqrt()) * normal - nr * view).normalize(),
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
      refraction_weight,
    ));

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
  if false {
    return (None, texture_loader);
  }
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
    let opt_f32_to_f64 = |o: Option<f32>| {
      if let Some(v) = o {
        Some(v as f64)
      } else {
        None
      }
    };

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

fn vecf32_to_point(v: [f32; 3]) -> Point {
  Vector::point(v[0] as f64, v[1] as f64, v[2] as f64)
}

pub fn load_scene(settings: &SceneSettings) -> Arc<Scene> {
  let mut scn = Scene::new(settings);
  {
    let scn = Arc::get_mut(&mut scn).unwrap();
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
            Triangle::new(material_index.unwrap_or(default_material), x, y, z)
          })
          .collect();
        object_triangles.append(&mut triangles);
      }

      let new_object = Box::new(Mesh::new(&object_triangles));
      scn.add_object(new_object);
    }
  }
  return scn;
}
