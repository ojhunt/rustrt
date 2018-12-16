use std::fmt::Debug;
use kdtree::HasPosition;
use bounding_box::BoundingBox;
use bounding_box::HasBoundingBox;
use colour::Colour;
use fragment::Fragment;
use heap::Heap;
use kdtree::KDTree;
use material::MaterialCollisionInfo;
use rand::{thread_rng, Rng};
use ray::Ray;
use scene::Scene;
use shader::LightSample;
use vectors::Vec4d;

#[derive(Clone, Debug)]
pub struct Photon {
  colour: Colour,
  position: Vec4d,
  direction: Vec4d,
}

impl HasBoundingBox for Photon {
  fn bounds(&self) -> BoundingBox {
    BoundingBox::new_from_point(self.position)
  }
}
impl HasPosition for Photon {
  fn get_position(&self) -> Vec4d {
    return self.position;
  }
}

#[derive(PartialEq)]
pub enum RecordMode {
  TerminatePath,
  DontRecord,
  RecordAndTerminate,
  Record,
}

impl RecordMode {
  pub fn should_record(&self) -> bool {
    match self {
      RecordMode::TerminatePath => false,
      RecordMode::DontRecord => false,
      _ => true,
    }
  }
  pub fn should_terminate(&self) -> bool {
    match self {
      RecordMode::TerminatePath => true,
      RecordMode::RecordAndTerminate => true,
      _ => false,
    }
  }
}

pub trait PhotonSelector: Debug + Clone {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode;
  fn weight_for_sample(&self, position: Vec4d, photon: &Photon, photon_count: usize, sample_radius: f64)
    -> Option<f64>;
}

#[derive(Debug)]
pub struct PhotonMap<Selector: PhotonSelector> {
  tree: KDTree<Photon>,
  selector: Selector,
}

fn random(min: f64, max: f64) -> f64 {
  thread_rng().gen_range(min, max)
}

fn random_in_hemisphere(normal: Vec4d) -> Vec4d {
  loop {
    let x = random(-1.0, 1.0);
    let y = random(-1.0, 1.0);
    let z = random(-1.0, 1.0);
    if (x * x + y * y + z * z) > 1.0 {
      continue;
    }
    let result = Vec4d::vector(x, y, z);
    if result.dot(normal) > 0.01 {
      return result;
    }
  }
}

impl<Selector: PhotonSelector> PhotonMap<Selector> {
  pub fn new(
    selector: &Selector,
    scene: &Scene,
    target_photo_count: usize,
    max_elements_per_leaf: usize,
  ) -> PhotonMap<Selector> {
    let mut photons: Vec<Photon> = vec![];
    let lights = &scene.get_lights();
    let mut virtual_lights: Vec<LightSample> = vec![];
    for light in lights {
      virtual_lights.append(&mut light.get_samples(1000, scene));
    }
    let mut bounces: usize = 0;
    let mut max_bounces: usize = 0;
    let mut paths = 0;
    let start = std::time::Instant::now();
    let mut photon_count = target_photo_count;
    while photons.len() < photon_count {
      'photon_loop: for sample in &virtual_lights {
        paths += 1;
        let mut light_dir = {
          let mut x;
          let mut y;
          let mut z;

          loop {
            x = random(-1.0, 1.0);
            y = random(-1.0, 1.0);
            z = random(-1.0, 1.0);
            if (x * x + y * y + z * z) <= 1.0 {
              break;
            }
          }

          Vec4d::vector(x, y, z).normalize() // this is a super awful/biased random, but whatever
        };

        if sample.direction.is_none() || light_dir.dot(sample.direction.unwrap()) < 0.01 {
          continue 'photon_loop;
        }

        let mut throughput = Colour::RGB(1.0, 1.0, 1.0);
        let mut photon_ray = Ray::new(sample.position + light_dir * 0.01, light_dir, None);
        let mut photon_colour = Colour::from(sample.diffuse);
        let mut path_length: usize = 0;
        'photon_bounce_loop: while photons.len() < photon_count && path_length < 16 {
          bounces += 1;
          path_length += 1;
          max_bounces = max_bounces.max(path_length);
          let (c, shadable) = match scene.intersect(&photon_ray) {
            None => continue 'photon_loop,
            Some(x) => x,
          };

          let fragment: Fragment = shadable.compute_fragment(scene, &photon_ray, &c);
          let material = match fragment.material {
            Some(inner) => scene.get_material(inner),
            None => continue 'photon_loop,
          };

          let surface: MaterialCollisionInfo = material.compute_surface_properties(scene, &photon_ray, &fragment);

          let mut next = {
            let mut selection = random(0.0, 1.0);
            let mut result: Option<(Ray, Colour)> = None;
            for (secondary_ray, secondary_colour, secondary_weight) in &surface.secondaries {
              if selection > *secondary_weight {
                selection -= secondary_weight;
                continue;
              }

              result = Some((secondary_ray.clone(), *secondary_colour))
            }
            result
          };
          let path_mode = selector.record_mode(&surface, bounces);
          let recorded_photon = if path_mode.should_record() {
            photons.push(Photon {
              colour: photon_colour * surface.diffuse_colour,
              position: fragment.position,
              direction: match &next {
                None => surface.normal,
                Some((ray, _)) => ray.direction,
              },
            });
            true
          } else {
            false
          };
          if path_mode.should_terminate() {
            if recorded_photon == false {
              // photon_count -= 1;
            }
            continue 'photon_loop;
          }
          if next.is_none() {
            let diffuse_direction = random_in_hemisphere(surface.normal);
            let diffuse_intensity = diffuse_direction.dot(surface.normal);

            let specular_direction = fragment.view.reflect(surface.normal);
            let specular_intensity = specular_direction.dot(fragment.view).powf(20.0);
            let mut next_colour;
            let new_direction = if random(0.0, diffuse_intensity + specular_intensity) < diffuse_intensity {
              next_colour = surface.diffuse_colour;
              random_in_hemisphere(surface.normal)
            } else {
              next_colour = surface.diffuse_colour;
              fragment.view.reflect(surface.normal)
            };

            next = Some((
              Ray::new(
                fragment.position + new_direction * 0.01,
                new_direction,
                Some(photon_ray.ray_context.clone()),
              ),
              next_colour,
            ));
          };

          let (next_ray, next_colour) = next.unwrap();

          // Now we know the colour and direction of the next bounce, let's decide if we're keeping it.
          throughput = throughput * next_colour;
          let p = random(0.0, 1.0);
          if p > throughput.intensity() {
            continue 'photon_loop;
          }
          throughput = throughput * (1.0 / p);
          photon_colour = photon_colour * next_colour;
          photon_ray = next_ray;
          continue 'photon_bounce_loop;
        }
      }
    }

    let end = std::time::Instant::now();

    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Time taken to generate photons: {}", time);
    let average_bounces = bounces as f64 / paths as f64;
    println!("Average path length: {}", average_bounces);
    println!("Total paths: {}", paths);
    println!("Max path length: {}", max_bounces);
    let tree = KDTree::new(&photons, max_elements_per_leaf);
    let (min, max) = tree.depth();
    println!("Tree minimum depth {}", min);
    println!("Tree maximum depth {}", max);

    return PhotonMap {
      tree,
      selector: selector.clone(),
    };
  }

  pub fn lighting(&self, position: Vec4d, direction: Vec4d, photon_samples: usize) -> Colour {
    if photon_samples == 0 {
      return Colour::RGB(0.0, 0.0, 0.0);
    }
    let mut result = Vec4d::new();
    let (photons, radius) = self.tree.nearest(position, photon_samples);

    for photon in &photons {
      if let Some(contribution) = self
        .selector
        .weight_for_sample(position, &photon, photons.len(), radius)
      {
        result = result + Vec4d::from(photon.colour) * contribution;
      }
    }
    return Colour::from(result);
  }
}

#[derive(Debug, Clone)]
pub struct DiffuseSelector {}

impl DiffuseSelector {
  pub fn new() -> DiffuseSelector {
    DiffuseSelector {}
  }
}

impl PhotonSelector for DiffuseSelector {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode {
    if depth > 1 {
      if surface.secondaries.len() > 0 {
        return RecordMode::DontRecord;
      }
      return RecordMode::Record;
    }
    return RecordMode::DontRecord;
  }

  fn weight_for_sample(
    &self,
    position: Vec4d,
    photon: &Photon,
    photon_count: usize,
    sample_radius: f64,
  ) -> Option<f64> {
    Some(1.0 / photon_count as f64)
  }
}

#[derive(Debug, Clone)]
pub struct CausticSelector {}

impl CausticSelector {
  pub fn new() -> CausticSelector {
    CausticSelector {}
  }
}

impl PhotonSelector for CausticSelector {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode {
    let mut secondary_weight = 0.0;
    for secondary in &surface.secondaries {
      secondary_weight += secondary.2;
    }
    if secondary_weight > 0.95 {
      return RecordMode::DontRecord;
    }
    if depth < 2 {
      return RecordMode::TerminatePath;
    }
    return RecordMode::Record;
  }
  fn weight_for_sample(
    &self,
    position: Vec4d,
    photon: &Photon,
    photon_count: usize,
    sample_radius: f64,
  ) -> Option<f64> {
    Some(1.0 / photon_count as f64)
  }
}
