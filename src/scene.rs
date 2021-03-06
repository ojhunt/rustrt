use crate::either::left;
use crate::media::Media;
use crate::material::compute_secondaries;
use crate::render_configuration::RenderConfiguration;
use std::collections::HashMap;
use crate::material::Material;
use crate::material::DefaultMaterial;
use crate::material::TransparentMaterial;
use crate::collision::Collision;
use crate::casefopen;
use crate::colour::Colour;
use crate::compound_object::CompoundObject;
use image::*;
use crate::intersectable::*;
use crate::material;
use crate::ray::Ray;
use crate::light::LightSample;
use crate::shader::Shadable;
use std::path::Path;
use std::path::PathBuf;
use crate::texture::Texture;
use crate::vectors::*;
use crate::photon_map::Timing;
use crate::either::Either;

#[derive(Debug, Copy, Clone)]
pub struct MaterialIdx(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct TextureIdx(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct NormalIdx(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct MediaIdx(pub u32);

impl MediaIdx {
  pub fn get<'a>(&self, s: &'a Scene) -> &'a Media {
    let MediaIdx(idx) = *self;
    return s.get_media(idx);
  }
}

impl NormalIdx {
  pub fn get(&self, s: &Scene) -> Vector {
    let NormalIdx(idx) = *self;
    return s.get_normal(idx);
  }
}

#[derive(Clone, Debug)]
pub struct SceneSettings {
  pub scene_file: String,
  pub camera_position: Point,
  pub camera_direction: Vector,
  pub camera_up: Vector,
  pub max_leaf_photons: usize,
  pub photon_samples: usize,
  pub width: usize,
  pub height: usize,
  pub samples_per_pixel: usize,
  pub photon_count: usize,
  pub use_direct_lighting: bool,
  pub use_multisampling: bool,
  pub gamma: f32,
}

impl SceneSettings {
  pub fn new() -> SceneSettings {
    return SceneSettings {
      scene_file: String::new(),
      camera_position: Vector::point(0., 0.5, 0.),
      camera_direction: Vector::vector(0., 0., 1.),
      camera_up: Vector::vector(0.0, 1.0, 0.0),
      max_leaf_photons: 8,
      width: 700,
      height: 700,
      photon_samples: 0,
      samples_per_pixel: 4,
      photon_count: 0,
      use_direct_lighting: false,
      use_multisampling: false,
      gamma: 1.0,
    };
  }
}

#[derive(Debug)]
pub struct Scene {
  settings: SceneSettings,
  path: PathBuf,
  pub directory: PathBuf,
  pub normals: Vec<Vector>,
  pub positions: Vec<Point>,
  pub materials: Vec<Box<material::Material>>,
  pub medias: Vec<Box<dyn Media>>,
  pub texture_coords: Vec<Vec2d>,
  pub textures: Vec<Texture>,
  material_map: HashMap<String, (MaterialIdx, bool)>,
  texture_map: HashMap<PathBuf, TextureIdx>,
  default_material: MaterialIdx,
  mirror_material: MaterialIdx,
  glass_material: MaterialIdx,
  root_object: CompoundObject,
}

impl Scene {
  pub fn new(settings: &SceneSettings) -> Scene {
    let real_path = Path::new(&settings.scene_file).canonicalize().unwrap();
    return Scene {
      settings: settings.clone(),
      path: real_path.clone(),
      directory: real_path.parent().unwrap().to_owned(),
      normals: Vec::new(),
      positions: Vec::new(),
      materials: vec![
        Box::new(DefaultMaterial::new(Colour::RGB(0.7, 0.7, 0.7), None)),
        Box::new(DefaultMaterial::new(Colour::RGB(1.0, 1.0, 1.0), Some(1.0))),
        Box::new(TransparentMaterial::new(2.5)),
      ],
      medias: Vec::new(),
      texture_coords: Vec::new(),
      textures: Vec::new(),
      root_object: CompoundObject::new(),
      material_map: HashMap::new(),
      texture_map: HashMap::new(),
      default_material: MaterialIdx(0),
      mirror_material: MaterialIdx(1),
      glass_material: MaterialIdx(2),
    };
  }

  pub fn load_texture(&mut self, file: &str, need_bumpmap: bool) -> Option<TextureIdx> {
    let resolved_path = self.directory.clone().join(file.replace("\\", "/"));
    if let Some(result) = self.texture_map.get(&resolved_path) {
      if need_bumpmap {
        let TextureIdx(idx) = result;
        self.textures[*idx as usize].generate_derivate_maps();
      }
      return Some(*result);
    }

    let format = if let Some(extension) = resolved_path.extension() {
      match (extension.to_str().unwrap()).to_lowercase().as_str() {
        "png" => ImageFormat::PNG,
        "pnm" => ImageFormat::PNG,
        "jpeg" => ImageFormat::JPEG,
        "jpg" => ImageFormat::JPEG,
        "tga" => ImageFormat::TGA,
        x => panic!("Extension {}", x),
      }
    } else {
      panic!();
    };

    let image = match casefopen::open(&resolved_path) {
      Ok(file) => {
        let buffer = std::io::BufReader::new(file);
        match image::load(buffer, format) {
          Ok(image) => image,
          Err(msg) => panic!("Failed to open {:?} with error: {}", resolved_path, msg),
        }
      }
      Err(msg) => panic!("Fopen({:?}) failed with {}", resolved_path, msg),
    };

    let texture = Texture::new(resolved_path.to_str().unwrap(), &image);

    let texture_idx = TextureIdx(self.textures.len() as u32);
    self.textures.push(texture);
    if need_bumpmap {
      let TextureIdx(idx) = texture_idx;
      self.textures[idx as usize].generate_derivate_maps();
    }
    self.texture_map.insert(resolved_path, texture_idx);
    return Some(texture_idx);
  }

  pub fn get_or_create_material<Loader: Fn(&mut Scene) -> Option<Box<Material>>>(
    &mut self,
    name: &str,
    loader: Loader,
  ) -> (MaterialIdx, bool) {
    if let Some(value) = self.material_map.get(name) {
      return *value;
    }
    if let Some(material) = loader(self) {
      let index = MaterialIdx(self.materials.len() as u32);
      let is_light = material.is_light();
      self.materials.push(material);
      self.material_map.insert(name.to_string(), (index, is_light));
      return (index, is_light);
    }
    return (MaterialIdx(0), false);
  }
  pub fn add_object(&mut self, object: Box<Intersectable>) {
    self.root_object.add_object(object)
  }
  pub fn default_material(&self) -> MaterialIdx {
    self.default_material
  }
  #[allow(dead_code)]
  pub fn mirror_material(&self) -> MaterialIdx {
    self.mirror_material
  }
  #[allow(dead_code)]
  pub fn glass_material(&self) -> MaterialIdx {
    self.glass_material
  }
  pub fn intersect<'a>(&'a self, ray: &Ray) -> Option<(Collision, Either<&'a Shadable, &'a Media>)> {
    if let Some(media) = ray.ray_context.media {
      let media = media.get(self);
    }
    return self
      .root_object
      .intersect(ray, HitMode::Nearest, ray.min, ray.max)
      .map(|(d, c)| (d, left(c)));
  }

  pub fn has_intersection(&self, ray: &Ray) -> bool {
    return self
      .root_object
      .intersect(ray, HitMode::AnyHit, ray.min, ray.max)
      .is_some();
  }

  pub fn finalize(&mut self) {
    Timing::time("Build scene graph", || {
      self.root_object.finalize();
    });
  }

  pub fn get_normal(&self, idx: u32) -> Vector {
    let n = self.normals[idx as usize];
    assert!(n.w() == 0.0);
    return n;
  }
  pub fn get_media<'a>(&'a self, idx: u32) -> &'a Media {
    return &*self.medias[idx as usize];
  }

  pub fn get_texture_coordinate(&self, idx: usize) -> Vec2d {
    let n = self.texture_coords[idx];
    return n;
  }

  pub fn get_material(&self, MaterialIdx(idx): MaterialIdx) -> &material::Material {
    return &*self.materials[idx as usize];
  }

  pub fn get_texture(&self, TextureIdx(idx): TextureIdx) -> &Texture {
    return &self.textures[idx as usize];
  }

  pub fn colour_and_depth_for_ray(&self, configuration: &RenderConfiguration, ray: &Ray) -> (Vector, f32) {
    return self.intersect_ray(configuration, ray, 0);
  }

  fn intersect_ray(&self, configuration: &RenderConfiguration, ray: &Ray, depth: usize) -> (Vector, f32) {
    if depth > 10 {
      return (Vector::vector(0.0, 0.0, 1.0), 0.0);
    }

    let (collision, shadable) = match self.intersect(ray) {
      None => return (Vector::new(), std::f32::INFINITY),
      Some((c, e)) => (c, e.unwrap_left()),
    };

    let (surface_colour, surface_distance) = {
      let fragment = shadable.compute_fragment(self, ray, &collision);

      let material = self.get_material(fragment.material);
      let surface = material.compute_surface_properties(self, ray, &fragment);

      // let ambient_colour = Vector::from(surface.ambient_colour);
      let mut diffuse_colour = Vector::from(surface.diffuse_colour);
      if let Some(emission) = surface.emissive_colour {
        return (
          Vector::from(
            emission.ambient * surface.ambient_colour
              + emission.diffuse * surface.diffuse_colour
              + emission.specular * surface.specular_colour,
          ),
          collision.distance,
        );
      }

      let mut colour;

      let mut max_secondary_distance = 0.0f32;
      let mut remaining_weight = 1.0;
      let mut secondaries_colour = Vector::new();
      for (ray, secondary_colour, weight) in compute_secondaries(ray, &fragment, &surface) {
        if remaining_weight <= 0.0 {
          break;
        }
        remaining_weight -= weight;
        let (secondary_intersection_colour, secondary_distance) = self.intersect_ray(configuration, &ray, depth + 1);
        secondaries_colour =
          secondaries_colour + Vector::from(Colour::from(secondary_intersection_colour) * secondary_colour * weight);
        max_secondary_distance = max_secondary_distance.max(secondary_distance);
      }
      colour = secondaries_colour;

      diffuse_colour = diffuse_colour * remaining_weight;
      if diffuse_colour.length() <= 0.01 {
        return (colour, collision.distance + max_secondary_distance);
      }
      let sample_lighting = configuration.lighting_integrator().lighting(self, &fragment, &surface);
      colour = colour
        + Vector::from(
          Colour::from(diffuse_colour) * sample_lighting.diffuse
            + Colour::from(surface.ambient_colour) * sample_lighting.ambient
            + Colour::from(surface.specular_colour) * sample_lighting.specular,
        );
      return (colour, collision.distance + max_secondary_distance);
    };
    return (surface_colour, surface_distance);
  }

  pub fn get_light_samples(&self, max_samples: usize) -> Vec<LightSample> {
    let light_objects = self.root_object.get_lights(self);
    let light_areas: &Vec<f32> = &light_objects.iter().map(|l| l.get_area()).collect();
    let total_area = {
      let mut area = 0.0;
      for light_area in light_areas {
        area += light_area;
      }
      area
    };
    let max_lights = max_samples;
    let mut remaining_lights = max_lights;
    let mut lights: Vec<LightSample> = vec![];
    for i in 0..light_areas.len() {
      let light_area = light_areas[i];
      let light_count = if i < light_areas.len() - 1 {
        (max_lights as f32 * (light_area / total_area)) as usize
      } else {
        remaining_lights
      };
      remaining_lights -= light_count;
      let mut samples = light_objects[i].get_samples(light_count, self);
      for mut sample in samples.iter_mut() {
        sample.weight *= light_count as f32 / max_lights as f32;
      }
      lights.append(&mut samples);
    }
    return lights;
  }
}
