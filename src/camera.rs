use ray::Ray;
use vectors::Vec4d;

pub trait Camera {
  fn get_rays(&self, width: usize, height: usize) -> Vec<(usize, usize, f64, Ray)>;
}

pub struct PerspectiveCamera {
  _width: usize,
  _height: usize,
  position: Vec4d,
  _direction: Vec4d,
  _up: Vec4d,
  _fov: f64,
  x_delta: Vec4d,
  y_delta: Vec4d,
  samples_per_pixel: usize,
  view_origin: Vec4d,
}

impl PerspectiveCamera {
  fn ray_for_coordinate(&self, x: f64, y: f64) -> Ray {
    let view_target = self.view_origin + (self.x_delta * x) - (self.y_delta * y);
    Ray::new(self.position, (view_target - self.position).normalize(), None)
  }
  pub fn new(
    width: usize,
    height: usize,
    position: Vec4d,
    target: Vec4d,
    up: Vec4d,
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

impl Camera for PerspectiveCamera {
  fn get_rays(&self, width: usize, height: usize) -> Vec<(usize, usize, f64, Ray)> {
    let mut result: Vec<(usize, usize, f64, Ray)> = Vec::new();
    for y in 0..height {
      for x in 0..width {
        if self.samples_per_pixel < 4 || true {
          result.push((x, y, 1.0, self.ray_for_coordinate(x as f64, y as f64)));
        } else {
          result.push((x, y, 0.25, self.ray_for_coordinate(x as f64 - 0.25, y as f64 - 0.25)));
          result.push((x, y, 0.25, self.ray_for_coordinate(x as f64 + 0.25, y as f64 - 0.25)));
          result.push((x, y, 0.25, self.ray_for_coordinate(x as f64 - 0.25, y as f64 + 0.25)));
          result.push((x, y, 0.25, self.ray_for_coordinate(x as f64 + 0.25, y as f64 + 0.25)));
        }
      }
    }
    return result;
  }
}
