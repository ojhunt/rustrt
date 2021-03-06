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
  fn render(&self, configuration: &Arc<RenderConfiguration>) -> RenderBuffer;
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

const DELTA: f32 = 0.1;
const MAX_DEPTH: u32 = 2;

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
    let sample_radius = radius / 4.0;
    let noise_radius = sample_radius / 2.0;;
    let positions = [
      (x - sample_radius, y - sample_radius),
      (x - sample_radius, y + sample_radius),
      // (x, y),
      (x + sample_radius, y - sample_radius),
      (x + sample_radius, y + sample_radius),
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
            x + 0.0 * random(-noise_radius, noise_radius),
            y + 0.0 * random(-noise_radius, noise_radius),
          ),
        )
      })
      .collect();

    let samples: Vec<((f64, f64), (Vector, f32))> = rays
      .iter()
      .map(|((x, y), r)| {
        let (c, d) = configuration.scene().colour_and_depth_for_ray(configuration, r);
        return ((*x, *y), (c.powf(self.gamma), d));
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
        let (value, distance, count) = if ((*a - average_colour).length() > DELTA && depth < MAX_DEPTH)
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

pub struct RenderBuffer {
  data: Vec<(Vector, usize, f64)>,
  pub width: usize,
  pub height: usize,
}

impl RenderBuffer {
  pub fn new(width: usize, height: usize) -> Self {
    let mut data = Vec::with_capacity(width * height);
    for _ in 0..width * height {
      data.push((Vector::new(), 0, std::f64::INFINITY));
    }
    return RenderBuffer { width, height, data };
  }
  pub fn get(&self, x: usize, y: usize) -> (Vector, usize, f64) {
    return self.data[y * self.width + x];
  }
  pub fn set(&mut self, x: usize, y: usize, sample: (Vector, usize, f64)) {
    self.data[y * self.width + x] = sample;
  }
  pub fn to_pixel_array(&self, gamma: f32) -> Vec<u8> {
    let stride = 3;
    let pitch = stride * self.width;
    let mut result = vec![255; pitch * self.height];
    for y in 0..self.height {
      let in_row_start = y * self.width;
      let out_row_start = y * pitch;
      for x in 0..self.width {
        let (value, sample_count, d) = self.data[x + in_row_start];
        let pixel_start = out_row_start + x * stride;
        let corrected_value: Vector = (value.powf(gamma) * 255.0f32).clamp32(0.0, 255.0);
        result[pixel_start + 0] = corrected_value.x().max(0.0).min(255.0) as u8;
        result[pixel_start + 1] = corrected_value.y().max(0.0).min(255.0) as u8;
        result[pixel_start + 2] = corrected_value.z().max(0.0).min(255.0) as u8;
      }
    }
    return result;
  }
}

impl Camera for PerspectiveCamera {
  fn render(&self, configuration: &Arc<RenderConfiguration>) -> RenderBuffer {
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
        buffer.set(
          x,
          y,
          (
            v.powf(self.gamma).clamp(Vector::splat(0.0), Vector::splat(1.0)),
            i,
            f as f64,
          ),
        );
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
                if (colour - sample_colour).length() > DELTA
                  || (sample_distance - distance).abs().sqrt() > 4.0 * DELTA as f64
                {
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
    return buffer;
  }
}
