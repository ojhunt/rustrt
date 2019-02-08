use std::sync::Arc;
use crate::photon_map::random;
use std::collections::HashMap;
use crate::material::Material;
use crate::material::DefaultMaterial;
use crate::collision::Collision;
use crate::casefopen;
use crate::colour::Colour;
use crate::compound_object::CompoundObject;
use image::*;
use crate::intersectable::Intersectable;
use crate::material;
use crate::photon_map::CausticSelector;
use crate::photon_map::DiffuseSelector;
use crate::photon_map::PhotonMap;
use crate::ray::Ray;
use crate::light::LightSample;
use crate::shader::Shadable;
use std::path::Path;
use std::path::PathBuf;
use crate::texture::Texture;
use crate::vectors::*;
use crate::photon_map::Timing;

#[derive(Debug, Copy, Clone)]
pub struct MaterialIdx(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct TextureIdx(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct NormalIdx(pub usize);

impl NormalIdx {
  pub fn get(&self, s: &Scene) -> Vector {
    let NormalIdx(idx) = *self;
    return s.get_normal(idx);
  }
}

#[derive(Clone, Debug)]
pub struct SceneSettings {
  pub output_file: String,
  pub scene_file: String,
  pub camera_position: Point,
  pub camera_target: Point,
  pub camera_up: Vector,
  pub max_leaf_photons: usize,
  pub photon_samples: usize,
  pub width: usize,
  pub height: usize,
  pub samples_per_pixel: usize,
  pub photon_count: usize,
  pub use_direct_lighting: bool,
  pub use_multisampling: bool,
}

impl SceneSettings {
  pub fn new() -> SceneSettings {
    return SceneSettings {
      output_file: String::new(),
      scene_file: String::new(),
      camera_position: Vector::point(0., 0.5, 0.),
      camera_target: Vector::point(0., 0., 10000000.),
      camera_up: Vector::vector(0.0, 1.0, 0.0),
      max_leaf_photons: 8,
      width: 700,
      height: 700,
      photon_samples: 0,
      samples_per_pixel: 4,
      photon_count: 0,
      use_direct_lighting: false,
      use_multisampling: false,
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
  pub texture_coords: Vec<Vec2d>,
  pub textures: Vec<Texture>,
  material_map: HashMap<String, (MaterialIdx, bool)>,
  texture_map: HashMap<PathBuf, TextureIdx>,
  _scene: CompoundObject,
  diffuse_photon_map: Option<PhotonMap<DiffuseSelector>>,
  caustic_photon_map: Option<PhotonMap<CausticSelector>>,
  light_samples: Vec<LightSample>,
}
struct SampleLighting {
  diffuse: Vector,
  ambient: Vector,
  specular: Vector,
}

impl SampleLighting {
  fn new() -> Self {
    SampleLighting {
      ambient: Vector::new(),
      diffuse: Vector::new(),
      specular: Vector::new(),
    }
  }
}
impl Scene {
  pub fn new(settings: &SceneSettings) -> Arc<Scene> {
    let real_path = Path::new(&settings.scene_file).canonicalize().unwrap();
    Arc::new(Scene {
      settings: settings.clone(),
      path: real_path.clone(),
      directory: real_path.parent().unwrap().to_owned(),
      normals: Vec::new(),
      positions: Vec::new(),
      materials: vec![Box::new(DefaultMaterial::new(Colour::RGB(0.7, 0.7, 0.7)))],
      texture_coords: Vec::new(),
      textures: Vec::new(),
      _scene: CompoundObject::new(),
      material_map: HashMap::new(),
      texture_map: HashMap::new(),
      diffuse_photon_map: None,
      caustic_photon_map: None,
      light_samples: vec![],
    })
  }

  pub fn load_texture(&mut self, file: &str, need_bumpmap: bool) -> Option<TextureIdx> {
    let resolved_path = self.directory.clone().join(file.replace("\\", "/"));
    if let Some(result) = self.texture_map.get(&resolved_path) {
      if need_bumpmap {
        let TextureIdx(idx) = result;
        self.textures[*idx].generate_derivate_maps();
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
        let buffer = std::io::BufReader::new(file);
        match image::load(buffer, format) {
          Ok(image) => image,
          Err(msg) => panic!("Failed to open {:?} with error: {}", resolved_path, msg),
        }
      }
      Err(msg) => panic!("Fopen({:?}) failed with {}", resolved_path, msg),
    };
    let texture = Texture::new(resolved_path.to_str().unwrap(), &image);

    let texture_idx = TextureIdx(self.textures.len());
    self.textures.push(texture);
    if need_bumpmap {
      let TextureIdx(idx) = texture_idx;
      self.textures[idx].generate_derivate_maps();
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
      let index = MaterialIdx(self.materials.len());
      let is_light = material.is_light();
      self.materials.push(material);
      self.material_map.insert(name.to_string(), (index, is_light));
      return (index, is_light);
    }
    return (MaterialIdx(0), false);
  }
  pub fn add_object(&mut self, object: Box<Intersectable>) {
    self._scene.add_object(object)
  }
  pub fn default_material(&self) -> MaterialIdx {
    MaterialIdx(0)
  }
  pub fn intersect<'a>(&'a self, ray: &Ray) -> Option<(Collision, &'a Shadable)> {
    return self._scene.intersect(ray, ray.min, ray.max);
  }

  pub fn finalize(this: &mut Arc<Self>, max_elements_per_leaf: usize) {
    Timing::time("Build scene graph", || {
      Arc::get_mut(this).unwrap()._scene.finalize();
    });
    Arc::get_mut(this).unwrap().light_samples = this.get_light_samples(10000);
    Self::rebuild_photon_map(this, max_elements_per_leaf);
  }

  pub fn get_normal(&self, idx: usize) -> Vector {
    let n = self.normals[idx];
    assert!(n.w() == 0.0);
    return n;
  }

  fn rebuild_photon_map(this: &mut Arc<Self>, max_elements_per_leaf: usize) {
    if this.settings.photon_count == 0 {
      return;
    }
    let diffuse_selector = Arc::new(DiffuseSelector::new(!this.settings.use_direct_lighting));
    Arc::get_mut(this).unwrap().diffuse_photon_map = PhotonMap::new(
      &diffuse_selector,
      this,
      &this.light_samples,
      this.settings.photon_count,
      max_elements_per_leaf,
    );
    let caustic_selector = Arc::new(CausticSelector::new());
    Arc::get_mut(this).unwrap().caustic_photon_map = PhotonMap::new(
      &caustic_selector,
      this,
      &this.light_samples,
      this.settings.photon_count,
      max_elements_per_leaf,
    );
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

  pub fn colour_and_depth_for_ray(&self, ray: &Ray, photon_samples: usize) -> (Vector, f64) {
    let lights = &self.light_samples;
    return self.intersect_ray(ray, lights, photon_samples, 0);
  }

  fn intersect_ray(&self, ray: &Ray, lights: &[LightSample], photon_samples: usize, depth: usize) -> (Vector, f64) {
    if depth > 10 {
      return (Vector::vector(1.0, 1.0, 1.0), 0.0);
    }
    let (collision, shadable) = match self.intersect(ray) {
      None => return (Vector::new(), std::f64::INFINITY),
      Some(collision) => collision,
    };

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
    let ambient_light = if let Some(ref diffuse_map) = self.diffuse_photon_map {
      let diffuse = diffuse_map.lighting(&fragment, &surface, photon_samples);
      let caustic = match &self.caustic_photon_map {
        None => Colour::RGB(0.0, 0.0, 0.0),
        Some(photon_map) => photon_map.lighting(&fragment, &surface, (photon_samples / 15).max(1)),
      };
      Some(diffuse + 0.0 * caustic)
    } else {
      None
    };

    let mut max_secondary_distance = 0.0f64;
    let mut remaining_weight = 1.0;
    let mut secondaries_colour = Vector::new();
    for (ray, secondary_colour, weight) in &surface.secondaries {
      if remaining_weight <= 0.0 {
        break;
      }
      remaining_weight -= weight;
      let (secondary_intersection_colour, secondary_distance) =
        self.intersect_ray(ray, lights, photon_samples, depth + 1);
      secondaries_colour =
        secondaries_colour + Vector::from(Colour::from(secondary_intersection_colour) * *secondary_colour * *weight);
      max_secondary_distance = max_secondary_distance.max(secondary_distance);
    }
    colour = secondaries_colour;
    Vector::new();
    diffuse_colour = diffuse_colour * remaining_weight;
    if diffuse_colour.length() <= 0.01 {
      return (colour, collision.distance + max_secondary_distance);
    }

    let direct_lighting = if self.settings.use_direct_lighting {
      let mut direct_lighting = SampleLighting::new();
      let light_samples = 50;
      let light_scale = lights.len() as f64 / light_samples as f64;
      for i in 0..light_samples {
        let light = &lights[random(0.0, lights.len() as f64) as usize];
        let mut ldir = light.position - surface.position;
        let ldir_len = ldir.length();
        ldir = ldir.normalize();
        let shadow_test = Ray::new_bound(surface.position, ldir, 0.005, ldir_len - 0.001, None);
        if self.intersect(&shadow_test).is_some() {
          continue;
        }

        let diffuse_intensity = light_scale * light.weight * ldir.dot(surface.normal).max(0.0);
        let ambient_intensity = light_scale * light.weight * light.ambient;
        direct_lighting.diffuse = direct_lighting.diffuse + light.diffuse * diffuse_intensity;
        direct_lighting.ambient = direct_lighting.ambient + light.ambient * ambient_intensity;
      }
      direct_lighting
    } else {
      SampleLighting::new()
    };

    colour = colour
      + Vector::from(
        Colour::from(diffuse_colour) * Colour::from(direct_lighting.diffuse)
          + Colour::from(surface.ambient_colour) * ambient_light.unwrap_or(Colour::from(direct_lighting.ambient))
          + Colour::from(surface.specular_colour) * Colour::from(direct_lighting.specular),
      );

    return (colour, collision.distance + max_secondary_distance);
  }

  pub fn get_light_samples(&self, max_samples: usize) -> Vec<LightSample> {
    let light_objects = self._scene.get_lights(self);
    let light_areas: &Vec<f64> = &light_objects.iter().map(|l| l.get_area()).collect();
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
        (max_lights as f64 * (light_area / total_area)) as usize
      } else {
        remaining_lights
      };
      remaining_lights -= light_count;
      let mut samples = light_objects[i].get_samples(light_count, self);
      for mut sample in samples.iter_mut() {
        sample.weight *= light_count as f64 / max_lights as f64;
      }
      lights.append(&mut samples);
    }
    return lights;
  }
}
