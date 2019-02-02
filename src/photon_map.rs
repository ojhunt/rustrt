use std::fmt::Debug;
use crate::kdtree::HasPosition;
use crate::bounding_box::BoundingBox;
use crate::bounding_box::HasBoundingBox;
use crate::colour::Colour;
use crate::fragment::Fragment;
use crate::kdtree::KDTree;
use crate::material::MaterialCollisionInfo;
use rand::{thread_rng, Rng};
use crate::ray::Ray;
use crate::scene::Scene;
use crate::shader::LightSample;
use crate::vectors::{Point, Vector};

#[derive(Clone, Debug)]
pub struct Photon {
  colour: Colour,
  position: Point,
  direction: Vector,
}

impl HasBoundingBox for Photon {
  fn bounds(&self) -> BoundingBox {
    BoundingBox::new_from_point(self.position)
  }
}
impl HasPosition for Photon {
  fn get_position(&self) -> Point {
    return self.position;
  }
}

#[derive(PartialEq)]
pub enum RecordMode {
  TerminatePath,
  DontRecord,
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
      _ => false,
    }
  }
}

pub trait PhotonSelector: Debug + Clone {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode;
  fn weight_for_sample(&self, position: Point, photon: &Photon, photon_count: usize, sample_radius: f64)
    -> Option<f64>;
}

#[derive(Debug)]
pub struct PhotonMap<Selector: PhotonSelector> {
  tree: KDTree<Photon>,
  selector: Selector,
}

pub fn random(min: f64, max: f64) -> f64 {
  thread_rng().gen_range(min, max)
}

fn random_in_hemisphere(normal: Vector) -> Vector {
  loop {
    let x = random(-1.0, 1.0);
    let y = random(-1.0, 1.0);
    let z = random(-1.0, 1.0);
    if (x * x + y * y + z * z) > 1.0 {
      continue;
    }
    let result = Vector::vector(x, y, z);
    if result.dot(normal) > 0.1 {
      return result;
    }
  }
}

impl<Selector: PhotonSelector> PhotonMap<Selector> {
  pub fn new(
    selector: &Selector,
    scene: &Scene,
    target_photon_count: usize,
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
    let mut actual_paths = 0;
    let start = std::time::Instant::now();
    while paths < target_photon_count {
      'photon_loop: for sample in &virtual_lights {
        if paths >= target_photon_count {
          break;
        }
        let light_dir = {
          if false {
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

            Vector::vector(x, y, z).normalize() // this is a super awful/biased random, but whatever
          } else {
            let u = random(0.0, 1.0);
            let v = 2.0 * 3.14127 * random(0.0, 1.0);
            Vector::vector(v.cos() * u.sqrt(), -(1.0 - u).sqrt(), v.sin() * u.sqrt())
          }
        };

        if sample.direction.is_none() || light_dir.dot(sample.direction.unwrap()) < 0.01 {
          continue 'photon_loop;
        }

        paths += 1;
        actual_paths += 1;

        let mut throughput = Colour::RGB(1.0, 1.0, 1.0);
        let mut photon_ray = Ray::new(sample.position + light_dir * 0.01, light_dir, None);
        let mut photon_colour = Colour::from(sample.emission) * 2.0;

        let mut path_length: usize = 0;
        let mut recorded = false;
        'photon_bounce_loop: while path_length < 64 {
          let current_colour = photon_colour;
          bounces += 1;
          path_length += 1;
          max_bounces = max_bounces.max(path_length);
          let (c, shadable) = match scene.intersect(&photon_ray) {
            None => {
              if !recorded {
                actual_paths += 1;
              }
              continue 'photon_loop;
            }
            Some(x) => x,
          };

          let fragment: Fragment = shadable.compute_fragment(scene, &photon_ray, &c);
          let material = scene.get_material(fragment.material);

          let surface: MaterialCollisionInfo = material.compute_surface_properties(scene, &photon_ray, &fragment);
          let mut remaining_weight = 1.0;
          let mut next = {
            let mut selection = random(0.0, 1.0);
            let mut result: Option<(Ray, Colour)> = None;
            for (_, _, secondary_weight) in &surface.secondaries {
              remaining_weight -= secondary_weight;
            }
            remaining_weight = remaining_weight.max(0.0);
            for (secondary_ray, secondary_colour, secondary_weight) in &surface.secondaries {
              if selection > *secondary_weight {
                selection -= secondary_weight;
                continue;
              }
              let next_colour = photon_colour * *secondary_colour;
              let next_power = next_colour.max_value() / photon_colour.max_value();
              result = Some((secondary_ray.clone(), next_colour * next_power))
            }
            result
          };

          if next.is_none() {
            let prob_diffuse = (surface.diffuse_colour * photon_colour).max_value() / photon_colour.max_value();
            let prob_specular = (surface.specular_colour * photon_colour).max_value() / photon_colour.max_value();
            let p = random(0.0, 1.0);
            let (new_direction, new_colour) = if p * remaining_weight < prob_diffuse {
              (
                random_in_hemisphere(surface.normal),
                surface.diffuse_colour * photon_colour * (1.0 / prob_diffuse),
              )
            } else if p * remaining_weight < (prob_diffuse + prob_specular) {
              (
                fragment.view.reflect(surface.normal),
                surface.specular_colour * photon_colour * (1.0 / prob_specular),
              )
            } else {
              continue 'photon_loop;
            };

            next = Some((
              Ray::new(
                fragment.position + new_direction * 0.001,
                new_direction,
                Some(photon_ray.ray_context.clone()),
              ),
              new_colour,
            ));
          };
          let (next_ray, next_colour) = next.unwrap();
          let path_mode = selector.record_mode(&surface, path_length);
          let recorded_photon = if path_mode.should_record() {
            if !recorded {
              actual_paths += 1;
              recorded = true;
            }

            photons.push(Photon {
              colour: current_colour,
              position: fragment.position,
              direction: photon_ray.direction,
            });
            true
          } else {
            false
          };
          if path_mode.should_terminate() {
            continue 'photon_loop;
          }
          if !recorded_photon {
            // Now we know the colour and direction of the next bounce, let's decide if we're keeping it.
            throughput = throughput * next_colour;
            let p = random(0.0, 1.0);
            if p > throughput.max_value() {
              continue 'photon_loop;
            }
            throughput = throughput * (1.0 / p);
          }
          photon_colour = next_colour;
          photon_ray = next_ray;
          continue 'photon_bounce_loop;
        }
      }
    }
    for i in 0..photons.len() {
      photons[i].colour = photons[i].colour * (1.0 / actual_paths as f64);
    }
    let end = std::time::Instant::now();

    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Time taken to generate photons: {}", time);
    let average_bounces = bounces as f64 / paths as f64;
    println!("Average path length: {}", average_bounces);
    println!("Total paths: {}", paths);
    println!("Total photon records: {}", photons.len());
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

  pub fn lighting(&self, surface: &MaterialCollisionInfo, photon_samples: usize) -> Colour {
    if photon_samples == 0 {
      return Colour::RGB(0.0, 0.0, 0.0);
    }
    let mut result = Vector::new();
    let radius_cutoff = 0.25 * 3.0;
    let (photons, radius) = self
      .tree
      .nearest(surface.position, photon_samples, radius_cutoff, |_p| true);
    let mut max_radius: f64 = 0.0;
    let _skipped = 0;
    for (photon, distance) in &photons {
      if let Some(contribution) = self
        .selector
        .weight_for_sample(surface.position, &photon, photons.len(), radius)
      {
        if *distance > radius_cutoff {
          continue;
        }
        let photon_position = photon.get_position();
        let to_photon = (photon_position - surface.position).normalize();
        let _cos_theta = to_photon.dot(surface.normal).abs();

        max_radius = max_radius.max(*distance);
        let weight = 1.0; //(1.0 - cos_theta); //1.0; //(-photon.direction.dot(surface.normal)).max(0.0); //* (radius_cutoff - distance).max(0.0) / radius_cutoff;
        result = result + Vector::from(photon.colour) * (contribution * weight).max(0.0);
      }
    }
    return Colour::from(result) * (1.0 / max_radius / max_radius / 3.1412).max(0.0);
  }
}

#[derive(Debug, Clone)]
pub struct DiffuseSelector {
  include_first_bounce: bool,
}

impl DiffuseSelector {
  pub fn new(include_first_bounce: bool) -> DiffuseSelector {
    DiffuseSelector { include_first_bounce }
  }
}

fn is_specular(surface: &MaterialCollisionInfo) -> bool {
  let mut secondary_weight = 0.0;
  for secondary in &surface.secondaries {
    secondary_weight += secondary.2;
  }
  if random(0.0, 1.0) < secondary_weight {
    return true;
  }
  return false;
}

impl PhotonSelector for DiffuseSelector {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode {
    if depth == 1 && is_specular(surface) {
      return RecordMode::TerminatePath;
    }

    if depth > 1 || self.include_first_bounce {
      return RecordMode::Record;
    }
    return RecordMode::DontRecord;
  }

  fn weight_for_sample(
    &self,
    _position: Point,
    _photon: &Photon,
    _photon_count: usize,
    _sample_radius: f64,
  ) -> Option<f64> {
    Some(1.0)
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
    if depth == 1 {
      if is_specular(surface) {
        return RecordMode::DontRecord;
      }
      return RecordMode::TerminatePath;
    }
    return RecordMode::Record;
  }
  fn weight_for_sample(
    &self,
    _position: Point,
    _photon: &Photon,
    _photon_count: usize,
    _sample_radius: f64,
  ) -> Option<f64> {
    Some(1.0) // / photon_count as f64)
  }
}
