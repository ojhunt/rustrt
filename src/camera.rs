use image::DynamicImage;
use image::ImageRgb8;
use scene::Scene;
use colour::Colour;
use photon_map::random;
use ray::Ray;
use vectors::{Point, Vector, VectorType};

struct RenderedImage {
  data: Vec<Colour>,
  width: usize,
  height: usize,
}

pub trait Camera {
  fn render(&self, scene: &Scene, photon_samples: usize) -> DynamicImage;
}

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
    let mut max_distance = 0.0f64;
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
  data: Vec<(Vector, f64)>,
  width: usize,
}

impl RenderBuffer {
  pub fn new(width: usize, height: usize) -> Self {
    let mut data = Vec::with_capacity(width * height);
    for _ in 0..width * height {
      data.push((Vector::new(), std::f64::INFINITY));
    }
    return RenderBuffer { width, data };
  }
  pub fn get(&self, x: usize, y: usize) -> (Vector, f64) {
    return self.data[y * self.width + x];
  }
  pub fn set(&mut self, x: usize, y: usize, sample: (Vector, f64)) {
    self.data[y * self.width + x] = sample;
  }
}

impl Camera for PerspectiveCamera {
  fn render(&self, scene: &Scene, photon_samples: usize) -> DynamicImage {
    let mut buffer = RenderBuffer::new(self._width, self._height);

    for x in 0..self._width {
      for y in 0..self._width {
        let ray = self.ray_for_coordinate(x as f64, y as f64);
        buffer.set(x, y, scene.colour_and_depth_for_ray(&ray, photon_samples));
      }
    }

    let mut future_samples = vec![];

    for x in 0..self._width {
      let minx = if x > 0 { -1i32 } else { 0 };
      let maxx = if x < self._width - 1 { 1 } else { 0 };
      'inner_loop: for y in 0..self._width {
        let miny = if y > 0 { -1i32 } else { 0 };
        let maxy = if y < self._height - 1 { 1 } else { 0 };
        let (sample_colour, sample_distance) = buffer.get(x, y);
        for i in minx..maxx {
          for j in miny..maxy {
            if i == 0 && j == 0 {
              continue;
            }
            let (colour, distance) = buffer.get((x as i32 + i) as usize, (y as i32 + j) as usize);
            if (colour - sample_colour).length() > DELTA || (sample_distance - distance).abs() > DELTA {
              future_samples.push((x, y));
              continue 'inner_loop;
            }
          }
        }
      }
    }

    println!("Resample count: {}", future_samples.len());
    let mut resample_count = 0;
    for (x, y) in future_samples {
      let (value, distance, sample_count) = self.multisample(scene, photon_samples, x as f64, y as f64, 1.0, 0);
      buffer.set(x, y, (value, distance));
      resample_count += sample_count;
    }

    println!("initial samples: {}", self._width * self._height);
    println!("resample count: {}", resample_count);

    let mut result = image::RgbImage::new(self._width as u32, self._height as u32);
    for (x, y, _pixel) in result.enumerate_pixels_mut() {
      let (value, _) = buffer.data[x as usize + y as usize * self._width];
      *_pixel = image::Rgb([
        (value.x() * 255.).max(0.).min(255.) as u8,
        (value.y() * 255.).max(0.).min(255.) as u8,
        (value.z() * 255.).max(0.).min(255.) as u8,
      ]);
    }

    return ImageRgb8(result);
  }
}
