use std::sync::Arc;
use image::DynamicImage;
use image::ImageRgb8;
use crate::scene::Scene;
use crate::photon_map::random;
use crate::ray::Ray;
use crate::vectors::{Point, Vector, VectorType};
use crate::dispatch_queue::DispatchQueue;

pub trait Camera: Clone {
  fn render(&self, scene: Arc<Scene>, photon_samples: usize) -> DynamicImage;
}

#[derive(Clone)]
pub struct PerspectiveCamera {
  _width: usize,
  _height: usize,
  position: Point,
  _direction: Vector,
  _up: Vector,
  _fov: f64,
  x_delta: Vector,
  y_delta: Vector,
  samples_per_pixel: usize,
  view_origin: Point,
}

const DELTA: f64 = 0.1;

impl PerspectiveCamera {
  fn ray_for_coordinate(&self, x: f64, y: f64) -> Ray {
    let view_target = self.view_origin + (self.x_delta * x) - (self.y_delta * y);
    Ray::new(self.position, (view_target - self.position).normalize(), None)
  }
  fn multisample(
    &self,
    scene: &Scene,
    photon_samples: usize,
    x: f64,
    y: f64,
    radius: f64,
    depth: u32,
  ) -> (Vector, f64, usize) {
    let noise_radius = if radius > 0.9 { 0.01 } else { radius / 2.0 };
    let positions = [
      (x - 0.25 * radius, y - 0.25 * radius),
      (x - 0.25 * radius, y + 0.25 * radius),
      (x + 0.25 * radius, y - 0.25 * radius),
      (x + 0.25 * radius, y + 0.25 * radius),
    ];
    let _max_distance = 0.0f64;
    let rays: Vec<((f64, f64), Ray)> = positions
      .iter()
      .map(|(x, y)| {
        (
          (*x, *y),
          self.ray_for_coordinate(
            x + random(-noise_radius, noise_radius),
            y + random(-noise_radius, noise_radius),
          ),
        )
      })
      .collect();

    let samples: Vec<((f64, f64), (Vector, f64))> = rays
      .iter()
      .map(|((x, y), r)| ((*x, *y), scene.colour_and_depth_for_ray(r, photon_samples)))
      .collect();
    let (average_colour, average_distance): (Vector, f64) = samples.iter().fold(
      (Vector::new(), 0.0),
      |(average_colour, average_distance), (_, (sample_colour, sample_distance))| {
        (
          average_colour + *sample_colour * 0.25,
          average_distance + sample_distance * 0.25,
        )
      },
    );
    return samples.iter().fold(
      (Vector::new(), 0.0f64, 0),
      |(current_value, current_max_distance, current_count), ((x, y), (a, distance))| {
        let (value, distance, count) =
          if ((*a - average_colour).length() > DELTA || (average_distance - distance).abs() > DELTA) && depth < 2 {
            let (v, distance, count) = self.multisample(scene, photon_samples, *x, *y, radius / 2.0, depth + 1);
            let one = Vector::splat(1.0);
            let mask = v.lt(one);
            (mask.select(v, one), distance.max(current_max_distance), count)
          } else {
            (*a, *distance, 4)
          };
        return (current_value + value * 0.25, distance, current_count + count);
      },
    );
  }

  pub fn new(
    width: usize,
    height: usize,
    position: Point,
    target: Point,
    up: Vector,
    fov: f64,
    samples_per_pixel: usize,
  ) -> PerspectiveCamera {
    let direction = (target - position).normalize();
    let right = direction.cross(up).normalize();

    // Technically we already have an "up" vector, but for out purposes
    // we need /true/ up, relative to our direction and right vectors.
    let up = right.cross(direction).normalize();

    let half_width = (fov.to_radians() / 2.).tan();
    let aspect_ratio = height as f64 / width as f64;
    let half_height = aspect_ratio * half_width;
    let view_origin = (position + direction) + up * half_height - right * half_width;

    let x_delta = (right * 2. * half_width) * (1. / width as f64);
    let y_delta = (up * 2. * half_height) * (1. / height as f64);

    return PerspectiveCamera {
      _width: width,
      _height: height,
      position,
      _direction: direction,
      view_origin,
      _up: up,
      _fov: fov,
      x_delta,
      y_delta,
      samples_per_pixel,
    };
  }
}

struct RenderBuffer {
  data: Vec<(Vector, usize, f64)>,
  width: usize,
}

impl RenderBuffer {
  pub fn new(width: usize, height: usize) -> Self {
    let mut data = Vec::with_capacity(width * height);
    for _ in 0..width * height {
      data.push((Vector::new(), 0, std::f64::INFINITY));
    }
    return RenderBuffer { width, data };
  }
  pub fn get(&self, x: usize, y: usize) -> (Vector, usize, f64) {
    return self.data[y * self.width + x];
  }
  pub fn set(&mut self, x: usize, y: usize, sample: (Vector, usize, f64)) {
    self.data[y * self.width + x] = sample;
  }
}

impl Camera for PerspectiveCamera {
  fn render(&self, scene: Arc<Scene>, photon_samples: usize) -> DynamicImage {
    let mut buffer = RenderBuffer::new(self._width, self._height);
    let mut first_sample_queue = DispatchQueue::new(10);
    for x in 0..self._width {
      for y in 0..self._height {
        let ray = self.ray_for_coordinate(x as f64, y as f64);
        first_sample_queue.add_task(&(x, y, ray));
      }
    }

    let start = std::time::Instant::now();

    {
      let result = {
        let scene = scene.clone();
        first_sample_queue.consume_tasks(&move |v| {
          let (x, y, ray) = v;
          let (colour, depth) = scene.colour_and_depth_for_ray(&ray, photon_samples);
          return (*x, *y, (colour, 1, depth));
        })
      };

      for (x, y, (v, i, f)) in result {
        buffer.set(x, y, (v.clamp(Vector::splat(0.0), Vector::splat(1.0)), i, f));
      }
    }

    let end = std::time::Instant::now();
    let delta = end - start;
    let time = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f64 / 1000.0;
    println!("Initial render time: {}s", time);
    let mut max_resample_count = 0;
    // let mut queue = Di
    if true {
      let mut multisample_queue = DispatchQueue::new(10);

      for x in 0..self._width {
        let minx = if x > 0 { -1i32 } else { 0 };
        let maxx = if x < self._width - 1 { 1 } else { 0 };
        'inner_loop: for y in 0..self._height {
          let miny = if y > 0 { -1i32 } else { 0 };
          let maxy = if y < self._height - 1 { 1 } else { 0 };
          let (sample_colour, _count, sample_distance) = buffer.get(x, y);
          for i in minx..maxx {
            for j in miny..maxy {
              if i == 0 && j == 0 {
                continue;
              }
              let (colour, _, distance) = buffer.get((x as i32 + i) as usize, (y as i32 + j) as usize);
              if (colour - sample_colour).length() > DELTA || (sample_distance - distance).abs() > DELTA {
                multisample_queue.add_task(&(x, y));
                continue 'inner_loop;
              }
            }
          }
        }
      }

      {
        let results = {
          let camera = self.clone();
          multisample_queue.consume_tasks(&move |(x, y)| {
            return (
              *x,
              *y,
              camera.multisample(&*scene, photon_samples, *x as f64, *y as f64, 1.0, 0),
            );
          })
        };

        let mut resample_count = 0;
        for (x, y, (colour, distance, count)) in results {
          resample_count += count;
          max_resample_count = max_resample_count.max(count);
          buffer.set(x, y, (colour, count, distance));
        }

        println!("initial samples: {}", self._width * self._height);
        println!("resample count: {}", resample_count);
      }
    }
    let mut result = image::RgbImage::new(self._width as u32, self._height as u32);
    for (x, y, _pixel) in result.enumerate_pixels_mut() {
      let (value, sample_count, _d) = buffer.data[x as usize + y as usize * self._width];
      if false {
        let proportion = sample_count as f64 / max_resample_count as f64;
        let d_colour = (proportion.sqrt() * 255.).max(0.).min(255.) as u8;
        *_pixel = image::Rgb([d_colour, d_colour, d_colour]);
      } else {
        *_pixel = image::Rgb([
          (value.x() * 255.).max(0.).min(255.) as u8,
          (value.y() * 255.).max(0.).min(255.) as u8,
          (value.z() * 255.).max(0.).min(255.) as u8,
        ]);
      }
    }

    return ImageRgb8(result);
  }
}
