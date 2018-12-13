use bounding_box::BoundingBox;
use bounding_box::HasBoundingBox;
use colour::Colour;
use fragment::Fragment;
use kdtree::KDTree;
use material::MaterialCollisionInfo;
use rand::{thread_rng, Rng};
use ray::Ray;
use scene::Scene;
use shader::LightSample;
use vectors::Vec4d;

#[derive(Clone, Debug)]
struct Photon {
  colour: Colour,
  position: Vec4d,
}

impl HasBoundingBox for Photon {
  fn bounds(&self) -> BoundingBox {
    BoundingBox::new_from_point(self.position)
  }
}

#[derive(Debug)]
pub struct PhotonMap {
  tree: KDTree<Photon>,
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

impl PhotonMap {
  pub fn new(scene: &Scene) -> PhotonMap {
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
    let photon_count = 1000000;
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
        let mut photon_colour = Colour::from(sample.specular);
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

          photons.push(Photon {
            colour: photon_colour * surface.diffuse_colour,
            position: fragment.position,
          });

          let mut next = {
            let mut selection = random(0.0, 1.0);
            let mut result = None;
            for (secondary_ray, secondary_colour, secondary_weight) in surface.secondaries {
              if selection > secondary_weight {
                selection -= secondary_weight;
                continue;
              }

              result = Some((secondary_ray, secondary_colour))
            }
            result
          };
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

          // Now we know the colour and diretion of the next bounce, let's decide if we're keeping it.
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
    return PhotonMap {
      tree: KDTree::new(&photons),
    };
  }
}
