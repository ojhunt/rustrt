use crate::render_configuration::RenderConfiguration;
use std::sync::Arc;
use image::DynamicImage;
use image::ImageRgb8;
use crate::photon_map::random;
use crate::ray::Ray;
use crate::vectors::{Point, Vector, VectorType};
use crate::dispatch_queue::DispatchQueue;
use crate::photon_map::Timing;

pub trait Camera: Sync + Send {
  fn render(&self, configuration: &Arc<RenderConfiguration>) -> DynamicImage;
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
  do_multisampling: bool,
  gamma: f32,
}

const DELTA: f32 = 0.2;

impl PerspectiveCamera {
  fn ray_for_coordinate(&self, x: f64, y: f64) -> Ray {
    let view_target = self.view_origin + (self.x_delta * x) - (self.y_delta * y);
    Ray::new(self.position, (view_target - self.position).normalize(), None)
  }
  fn multisample(
    &self,
    configuration: &RenderConfiguration,
    x: f64,
    y: f64,
    radius: f64,
    depth: u32,
  ) -> (Vector, f32, usize) {
    let noise_radius = if radius > 0.9 { 0.01 } else { radius / 2.0 };
    let positions = [
      (x - 0.25 * radius, y - 0.25 * radius),
      (x - 0.25 * radius, y + 0.25 * radius),
      (x, y),
      (x + 0.25 * radius, y - 0.25 * radius),
      (x + 0.25 * radius, y + 0.25 * radius),
    ];
    let subsample_count = positions.len();
    let subsample_weight = 1.0 / subsample_count as f32;
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

    let samples: Vec<((f64, f64), (Vector, f32))> = rays
      .iter()
      .map(|((x, y), r)| {
        (
          (*x, *y),
          configuration.scene().colour_and_depth_for_ray(configuration, r),
        )
      })
      .collect();
    let (average_colour, average_distance): (Vector, f32) = samples.iter().fold(
      (Vector::new(), 0.0),
      |(average_colour, average_distance), (_, (sample_colour, sample_distance))| {
        (
          average_colour + *sample_colour * subsample_weight,
          average_distance + sample_distance * subsample_weight,
        )
      },
    );
    return samples.iter().fold(
      (Vector::new(), 0.0f32, 0),
      |(current_value, current_max_distance, current_count), ((x, y), (a, distance))| {
        let (value, distance, count) = if ((*a - average_colour).length() > DELTA && depth < 2)
          || ((average_distance - distance).abs() > DELTA && depth < 2)
        {
          let (v, distance, count) = self.multisample(configuration, *x, *y, radius / 2.0, depth + 1);
          let one = Vector::splat(1.0);
          let mask = v.lt(one);
          (mask.select(v, one), distance.max(current_max_distance), count)
        } else {
          (*a, *distance, subsample_count)
        };
        return (
          current_value + value * subsample_weight,
          distance,
          current_count + count,
        );
      },
    );
  }

  pub fn new(
    width: usize,
    height: usize,
    position: Point,
    direction: Vector,
    up: Vector,
    fov: f64,
    samples_per_pixel: usize,
    do_multisampling: bool,
    gamma: f32,
  ) -> PerspectiveCamera {
    let direction = direction.normalize();
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
      do_multisampling,
      gamma,
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
  fn render(&self, configuration: &Arc<RenderConfiguration>) -> DynamicImage {
    let mut buffer = RenderBuffer::new(self._width, self._height);
    let mut first_sample_queue = DispatchQueue::default();
    {
      let _t = Timing::new("Generating first sample set");
      for x in 0..self._width {
        for y in 0..self._height {
          let ray = self.ray_for_coordinate(x as f64, y as f64);
          first_sample_queue.add_task(&(x, y, ray));
        }
      }
    }

    {
      let result = {
        let _t = Timing::new("First render pass");
        let configuration = configuration.clone();
        first_sample_queue.consume_tasks(&move |v| {
          let (x, y, ray) = v;
          let (colour, depth) = configuration.scene().colour_and_depth_for_ray(&configuration, &ray);
          return (*x, *y, (colour, 1, depth));
        })
      };

      let _t = Timing::new("Copy first render results");
      for (x, y, (v, i, f)) in result {
        buffer.set(x, y, (v.clamp(Vector::splat(0.0), Vector::splat(1.0)), i, f as f64));
      }
    }

    let mut max_resample_count = 0;
    // let mut queue = Di
    if self.do_multisampling {
      let mut multisample_queue = DispatchQueue::default();
      {
        let _t = Timing::new("Performing initial multisample tasks");
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
                if (colour - sample_colour).length() > DELTA || (sample_distance - distance).abs() > DELTA as f64 {
                  multisample_queue.add_task(&(x, y));
                  continue 'inner_loop;
                }
              }
            }
          }
        }
      }

      {
        let results = {
          let _t = Timing::new("Multisampling");
          let camera = self.clone();
          let configuration = configuration.clone();
          multisample_queue.consume_tasks(&move |(x, y)| {
            return (*x, *y, camera.multisample(&configuration, *x as f64, *y as f64, 1.0, 0));
          })
        };

        {
          let _t = Timing::new("Copying resample output");
          for (x, y, (colour, distance, count)) in results {
            max_resample_count = max_resample_count.max(count);
            buffer.set(x, y, (colour, count, distance.into()));
          }
        }
      }
    }
    let _t = Timing::new("Creating output image");
    let mut result = image::RgbImage::new(self._width as u32, self._height as u32);
    let mut max_d = 0.0;

    for (x, y, _pixel) in result.enumerate_pixels() {
      let (_, _, d) = buffer.data[x as usize + y as usize * self._width];
      max_d = d.max(max_d);
    }
    println!("Max depth: {}", max_d);
    for (x, y, _pixel) in result.enumerate_pixels_mut() {
      let (value, sample_count, d) = buffer.data[x as usize + y as usize * self._width];
      if false {
        let d_colour = (d / max_d * 255.).max(0.).min(255.) as u8;
        *_pixel = image::Rgb([d_colour, d_colour, d_colour]);
      } else if false {
        let proportion = sample_count as f64 / max_resample_count as f64;
        let d_colour = (proportion.sqrt() * 255.).max(0.).min(255.) as u8;
        *_pixel = image::Rgb([d_colour, d_colour, d_colour]);
      } else if true {
        let clamped = value.clamp(Vector::splat(0.0), Vector::splat(std::f32::MAX));

        *_pixel = image::Rgb([
          (clamped.x().powf(1.0 / self.gamma) * 255.).max(0.).min(255.) as u8,
          (clamped.y().powf(1.0 / self.gamma) * 255.).max(0.).min(255.) as u8,
          (clamped.z().powf(1.0 / self.gamma) * 255.).max(0.).min(255.) as u8,
        ]);
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
