use camera::Camera;
use collision::Collision;
use colour::Colour;
use compound_object::CompoundObject;
use image::*;
use intersectable::Intersectable;
use material;
use photon_map::CausticSelector;
use photon_map::DiffuseSelector;
use photon_map::PhotonMap;
use rand::{thread_rng, Rng};
use ray::Ray;
use shader::Light;
use shader::LightSample;
use shader::Shadable;
use std::path::Path;
use std::path::PathBuf;
use texture::Texture;
use vectors::{Vec2d, Vec4d};

#[derive(Debug, Copy, Clone)]
pub struct MaterialIdx(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct TextureIdx(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct NormalIdx(pub usize);

impl NormalIdx {
  pub fn get(&self, s: &Scene) -> Vec4d {
    let NormalIdx(idx) = *self;
    return s.get_normal(idx);
  }
}

#[derive(Clone, Debug)]
pub struct SceneSettings {
  pub output_file: String,
  pub scene_file: String,
  pub camera_position: Vec4d,
  pub camera_target: Vec4d,
  pub camera_up: Vec4d,
  pub max_leaf_photons: usize,
  pub photon_samples: usize,
  pub width: usize,
  pub height: usize,
  pub samples_per_pixel: usize,
  pub photon_count: usize,
}

impl SceneSettings {
  pub fn new() -> SceneSettings {
    return SceneSettings {
      output_file: String::new(),
      scene_file: String::new(),
      camera_position: Vec4d::point(0., 0.5, 0.),
      camera_target: Vec4d::point(0., 0., 10000000.),
      camera_up: Vec4d::vector(0.0, 1.0, 0.0),
      max_leaf_photons: 8,
      width: 700,
      height: 700,
      photon_samples: 0,
      samples_per_pixel: 4,
      photon_count: 0,
    };
  }
}

#[derive(Debug)]
pub struct Scene {
  settings: SceneSettings,
  path: PathBuf,
  pub directory: PathBuf,
  pub normals: Vec<Vec4d>,
  pub positions: Vec<Vec4d>,
  pub materials: Vec<Box<material::Material>>,
  pub texture_coords: Vec<Vec2d>,
  pub textures: Vec<Texture>,
  _scene: CompoundObject,
  diffuse_photon_map: Option<PhotonMap<DiffuseSelector>>,
  caustic_photon_map: Option<PhotonMap<CausticSelector>>,
}

impl Scene {
  pub fn new(settings: &SceneSettings) -> Scene {
    let real_path = Path::new(&settings.scene_file).canonicalize().unwrap();
    Scene {
      settings: settings.clone(),
      path: real_path.clone(),
      directory: real_path.parent().unwrap().to_owned(),
      normals: Vec::new(),
      positions: Vec::new(),
      materials: Vec::new(),
      texture_coords: Vec::new(),
      textures: Vec::new(),
      _scene: CompoundObject::new(),
      diffuse_photon_map: None,
      caustic_photon_map: None,
    }
  }
  pub fn add_object(&mut self, object: Box<Intersectable>) {
    self._scene.add_object(object)
  }

  pub fn intersect<'a>(&'a self, ray: &Ray) -> Option<(Collision, &'a Shadable)> {
    return self._scene.intersect(ray, ray.min, ray.max);
  }

  pub fn finalize(&mut self, max_elements_per_leaf: usize) {
    self._scene.finalize();
    self.rebuild_photon_map(max_elements_per_leaf);
  }

  pub fn get_normal(&self, idx: usize) -> Vec4d {
    let n = self.normals[idx];
    assert!(n.w == 0.0);
    return n;
  }

  fn rebuild_photon_map(&mut self, max_elements_per_leaf: usize) {
    let diffuse_selector = DiffuseSelector::new();
    self.diffuse_photon_map = Some(PhotonMap::new(
      &diffuse_selector,
      self,
      self.settings.photon_count,
      max_elements_per_leaf,
    ));
    let caustic_selector = CausticSelector::new();
    self.caustic_photon_map = Some(PhotonMap::new(&caustic_selector, self, 2000, max_elements_per_leaf));
  }

  pub fn get_texture_coordinate(&self, idx: usize) -> Vec2d {
    let n = self.texture_coords[idx];
    return n;
  }
  pub fn get_lights<'a>(&'a self) -> Vec<&'a Light> {
    return self._scene.get_lights(self);
  }
  pub fn get_material(&self, MaterialIdx(idx): MaterialIdx) -> &material::Material {
    return &*self.materials[idx];
  }

  pub fn get_texture(&self, TextureIdx(idx): TextureIdx) -> &Texture {
    return &self.textures[idx];
  }

  fn intersect_ray(&self, ray: &Ray, lights: &Vec<Vec4d>, photon_samples: usize, depth: usize) -> Vec4d {
    if depth > 10 {
      return Vec4d::vector(1.0, 1.0, 1.0);
    }
    match self.intersect(ray) {
      None => return Vec4d::new(),
      Some((c, shadable)) => {
        let fragment = shadable.compute_fragment(self, ray, &c);

        let material = match fragment.material {
          Some(inner) => self.get_material(inner),
          None => return Vec4d::new(),
        };
        let surface = material.compute_surface_properties(self, ray, &fragment);
        // let ambient_colour = Vec4d::from(surface.ambient_colour);
        let mut diffuse_colour = Vec4d::from(surface.diffuse_colour);
        if let Some(c) = surface.emissive_colour {
          return Vec4d::from(c);
        }

        let mut colour = Vec4d::new();
        let ambient_light = {
          let diffuse = match &self.diffuse_photon_map {
            None => Colour::RGB(0.0, 0.0, 0.0),
            Some(photon_map) => (photon_map.lighting(&surface, photon_samples)),
          };
          // let caustic = match &self.caustic_photon_map {
          //   None => Colour::RGB(0.0, 0.0, 0.0),
          //   Some(photon_map) => (photon_map.lighting(&surface, photon_samples)),
          // };
          (diffuse) //+ caustic)
        };
        return Vec4d::from(ambient_light);
        if true {
          let mut remaining_weight = 1.0;
          for (ray, secondary_colour, weight) in &surface.secondaries {
            if remaining_weight <= 0.0 {
              break;
            }
            remaining_weight -= weight;
            colour = colour
              + Vec4d::from(
                Colour::from(self.intersect_ray(ray, lights, photon_samples, depth + 1)) * *secondary_colour * *weight,
              );
          }
          let diffuse_colour = diffuse_colour * remaining_weight;
          colour = Vec4d::from(Colour::from(diffuse_colour) * ambient_light); //Vec4d::from(ambient_light); //colour + Vec4d::from(surface.diffuse_colour * remaining_weight * ambient_colour);
          if true || diffuse_colour.length() <= 0.01 {
            return colour;
          }
          let light_samples = 8;
          let mut has_intersected = false;
          for i in 0..light_samples {
            let light = &lights[thread_rng().gen_range(0, lights.len())];
            let mut ldir = *light - surface.position;
            let ldir_len = ldir.dot(ldir).sqrt();
            ldir = ldir.normalize();
            if i * 2 < light_samples || has_intersected {
              let shadow_test = Ray::new_bound(surface.position, ldir, 0.01 * ldir_len, ldir_len * 0.999, None);

              if self.intersect(&shadow_test).is_some() {
                has_intersected = true;
                continue;
              }
            }
            let diffuse_intensity = ldir.dot(surface.normal) / light_samples as f64;
            if diffuse_intensity <= 0.0 || true {
              continue;
            }

            colour = colour + diffuse_colour * diffuse_intensity;
          }
        } else {
          colour = diffuse_colour;
          // Vec4d::vector(diffuse_colour.x, 1. - c.distance.log10() / 2., 0.0); // ambient_colour + diffuse_colour;
        }
        return colour;
      }
    }
  }
  pub fn get_light_samples(&self, max_samples: usize) -> Vec<LightSample> {
    let light_objects = &self._scene.get_lights(self);
    let light_areas: &Vec<f64> = &light_objects.iter().map(|l| l.get_area()).collect();
    let total_area = {
      let mut area = 0.0;
      for light_area in light_areas {
        area += light_area;
      }
      area
    };
    if light_objects.len() != 0 {
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
        lights.append(&mut samples);
      }
      return lights;
    } else {
      return (vec![
        Vec4d::point(2., 3., 0.),
        Vec4d::point(-10., -12., -4.),
        Vec4d::point(-16., 9.5, 4.),
        Vec4d::point(-14., 19.5, -2.),
      ])
      .iter()
      .map(|p| LightSample {
        position: *p,
        direction: None,
        diffuse: Vec4d::vector(1.0, 1.0, 1.0),
        specular: Vec4d::vector(1.0, 1.0, 1.0),
        emission: Vec4d::vector(0.0, 10.0, 10.0),
        weight: 0.25,
      })
      .collect();
    };
  }
  pub fn render<C: Camera>(&self, camera: &C, photon_samples: usize, width: usize, height: usize) -> DynamicImage {
    let mut result = image::RgbImage::new(width as u32, height as u32);
    let mut buffer: Vec<Vec<Vec4d>> = vec![];
    for _ in 0..(width * height) {
      buffer.push(vec![]);
    }
    let rays = camera.get_rays(width, height);

    let lights = self.get_light_samples(10000).iter().map(|l| l.position).collect();
    println!("photon samples: {}", photon_samples);
    let iteration_count = 1;
    for _ in 0..iteration_count {
      for (x, y, pixel_contribution_weight, ray) in &rays {
        let colour = self.intersect_ray(ray, &lights, photon_samples, 0);
        buffer[x + y * width].push(colour * (*pixel_contribution_weight / iteration_count as f64));
      }
    }

    for (x, y, _pixel) in result.enumerate_pixels_mut() {
      let mut r: f64 = 0.0;
      let mut g: f64 = 0.0;
      let mut b: f64 = 0.0;
      for v in &buffer[x as usize + y as usize * width] {
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
