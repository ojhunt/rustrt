use std::sync::Arc;
use std::fmt::Debug;
use std::time::Instant;
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
use crate::light::LightSample;
use crate::vectors::{Point, Vector,VectorType};
use crate::dispatch_queue::DispatchQueue;

#[derive(Clone, Debug, Copy)]
pub struct Photon {
  colour: Colour,
  position: Point,
  in_direction: Vector,
  out_direction: Vector,
  is_direct: bool,
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

pub trait PhotonSelector: Debug + Clone + Sync + Send {
  fn record_mode(&self, surface: &MaterialCollisionInfo, depth: usize) -> RecordMode;
  fn weight_for_sample(&self, position: Point, photon: &Photon, photon_count: usize, sample_radius: f64)
    -> Option<f64>;
}

#[derive(Debug)]
pub struct PhotonMap<Selector: PhotonSelector + 'static> {
  tree: KDTree<Photon>,
  selector: Arc<Selector>,
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
fn make_photon(sample: &LightSample) -> (Ray, Colour) {
  loop {
    let mut light_dir = {
      let u = random(0.0, 1.0);
      let v = 2.0 * 3.14127 * random(0.0, 1.0);
      Vector::vector(v.cos() * u.sqrt(), -(1.0 - u).sqrt(), v.sin() * u.sqrt())
    };

    // if sample.direction.is_some() && light_dir.dot(sample.direction.unwrap()) < 0.0 {
    //   light_dir = -light_dir;
    // }

    return (
      Ray::new(sample.position + light_dir * 0.01, light_dir, None),
      Colour::from(
        sample.emission.diffuse * sample.diffuse
          + sample.emission.ambient * sample.ambient
          + sample.emission.specular * sample.specular,
      ),
    );
  }
}

fn bounce_photon<Selector: PhotonSelector + 'static>(
  selector: &Arc<Selector>,
  scene: &Arc<Scene>,
  initial_ray: &Ray,
  initial_colour: Colour,
) -> Vec<Photon> {
  let mut throughput = Colour::RGB(1.0, 1.0, 1.0);
  let mut path_length: usize = 0;
  let mut recorded = false;
  let mut paths = 0;
  let mut actual_paths = 0;
  let mut max_bounces = 0;
  let mut bounces = 0;
  let mut photons = vec![];
  let mut photon_colour = initial_colour;
  let mut photon_ray = initial_ray.clone();
  // println!("Photon colour {:?}", photon_colour);
  while path_length < 256 {
    let current_colour = photon_colour;

    paths += 1;
    actual_paths += 1;
    bounces += 1;
    path_length += 1;
    max_bounces = max_bounces.max(path_length);
    let (c, shadable) = match scene.intersect(&photon_ray) {
      None => {
        break;
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
        break;
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
    let (next_ray, mut next_colour) = next.unwrap();
    let path_mode = selector.record_mode(&surface, path_length);
    let recorded_photon = if path_mode.should_record() {
      if !recorded {
        actual_paths += 1;
        recorded = true;
      }

      photons.push(Photon {
        colour: current_colour,
        position: fragment.position,
        in_direction: photon_ray.direction,
        out_direction: next_ray.direction,
        is_direct: path_length == 1,
      });
      true
    } else {
      false
    };
    if path_mode.should_terminate() {
      break;
    }
    if !recorded_photon {
      // Now we know the colour and direction of the next bounce, let's decide if we're keeping it.
      throughput = throughput * next_colour;
      let p = random(0.0, 1.0);
      if p > throughput.max_value().sqrt().sqrt() {
        break;
      }
      throughput = throughput * (1.0 / p);
      next_colour = next_colour;
    }
    photon_colour = next_colour;
    photon_ray = next_ray;
  }
  return photons;
}

fn bounce_photons<Selector: PhotonSelector + 'static>(
  selector: &Arc<Selector>,
  scene: &Arc<Scene>,
  initial_photons: &[(Ray, Colour)],
) -> Vec<Photon> {
  let mut photons = vec![];
  let mut queue = DispatchQueue::new(8);

  'photon_loop: for photon in initial_photons {
    queue.add_task(photon);
  }

  let scene = scene.clone();
  let selector = selector.clone();
  queue
    .consume_tasks(&move |(photon_ray, photon_colour)| {
      return bounce_photon(&selector, &scene, photon_ray, *photon_colour);
    })
    .iter()
    .for_each(|photon_paths| {
      photons.reserve(photon_paths.len());
      for photon in photon_paths {
        photons.push(*photon);
      }
    });

  return photons;
}

pub struct Timing {
  name: String,
  start: Instant,
}
impl Timing {
  #[must_use]
  pub fn new(name: &str) -> Timing {
    Timing {
      name: name.to_string(),
      start: Instant::now(),
    }
  }
  pub fn time<F, T>(name: &str, mut f: F) -> T
  where
    F: FnMut() -> T,
  {
    let _t = Timing::new(name);
    let r = f();
    return r;
  }
}
impl Drop for Timing {
  fn drop(&mut self) {
    println!("{} took {:?}ms ", self.name, (Instant::now() - self.start).as_millis());
  }
}

impl<Selector: PhotonSelector + 'static> PhotonMap<Selector> {
  pub fn new(
    selector: &Arc<Selector>,
    scene: &Arc<Scene>,
    lights: &[LightSample],
    target_photon_count: usize,
    max_elements_per_leaf: usize,
  ) -> Option<PhotonMap<Selector>> {
    assert!(!lights.is_empty());
    let start = std::time::Instant::now();

    let initial_photons = Timing::time("Generating initial rays", || {
      let mut initial_photons = vec![];
      let total_power = lights.iter().fold(0.0, |a, b| a + b.output());
      for light in lights {
        let power = light.output();
        let photon_count = (power / total_power * target_photon_count as f64).ceil() as usize;
        for _ in 0..photon_count.max(1) {
          initial_photons.push(make_photon(&light));
        }
      }
      return initial_photons;
    });

    let initial_photon_count = initial_photons.len();
    assert_eq!(initial_photon_count, initial_photons.len());
    let mut photons = Timing::time("Bouncing photons", || {
      return bounce_photons(selector, scene, &initial_photons);
    });
    if photons.is_empty() {
      return None;
    }
    {
      let _t = Timing::new("Normalising photon power");
      for i in 0..photons.len() {
        photons[i].colour = photons[i].colour * (1.0 / initial_photon_count as f64);
      }
    }
    let tree = Timing::time("Creating KDTree", || {
      return KDTree::new(&photons, max_elements_per_leaf);
    });

    return Some(PhotonMap {
      tree,
      selector: selector.clone(),
    });
  }

  pub fn lighting(&self, _fragment: &Fragment, surface: &MaterialCollisionInfo, photon_samples: usize) -> Colour {
    if photon_samples == 0 {
      return Colour::RGB(0.0, 0.0, 0.0);
    }
    let mut result = Vector::new();
    let surface_normal = surface.normal;
    let position = surface.position;
    let _radius_cutoff = 0.25 * 30.0;

    let (photons, radius) = self.tree.nearest(surface.position, photon_samples, |p| {
      // if p.out_direction.dot(surface_normal) < 0.0 {
      //   return None;
      // }

      let to_vector = p.position - position;

      let length = to_vector.length();

      let _overlap = (to_vector / length).dot(surface_normal);
      // if overlap > 0.4 {
      //   return None;
      // }
      return Some(to_vector.length());
    });
    if photons.len() == 0 {
      return Colour::RGB(0.0, 0.0, 0.0);
    }
    let mut max_radius: f64 = 0.0;
    let _skipped = 0;
    for (photon, distance) in &photons {
      if let Some(contribution) = self
        .selector
        .weight_for_sample(surface.position, &photon, photons.len(), radius)
      {
        max_radius = max_radius.max(*distance);
        let weight = photon.in_direction.dot(-surface_normal).max(0.0);
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
    if depth == 1 && is_specular(surface) && false {
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
