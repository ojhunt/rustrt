use ray::Ray;
use vectors::Vec4d;

pub trait Camera {
    fn get_rays(&self, width: usize, height: usize) -> Vec<(usize, usize, f64, Ray)>;
    fn get_differentials(&self, r: &Ray) -> (Ray, Ray);
}

pub struct PerspectiveCamera {
    width: usize,
    height: usize,
    position: Vec4d,
    direction: Vec4d,
    up: Vec4d,
    fov: f64,
    x_delta: Vec4d,
    y_delta: Vec4d,
    view_origin: Vec4d,
    dxDifferential: Ray,
    dyDifferential: Ray,
}

impl PerspectiveCamera {
    fn ray_for_coordinate(&self, x: f64, y: f64) -> Ray {
        let view_target = self.view_origin + (self.x_delta * x) - (self.y_delta * y);
        Ray::new(
            self.position,
            (view_target - self.position).normalize(),
            None,
        )
    }
    pub fn new(
        width: usize,
        height: usize,
        position: Vec4d,
        target: Vec4d,
        up: Vec4d,
        fov: f64,
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
            width,
            height,
            position,
            direction,
            view_origin,
            up,
            fov,
            x_delta,
            y_delta,
            dxDifferential: Ray::new(position, x_delta, None),
            dyDifferential: Ray::new(position, y_delta, None),
        };
    }
}

impl Camera for PerspectiveCamera {
    fn get_rays(&self, width: usize, height: usize) -> Vec<(usize, usize, f64, Ray)> {
        let mut result: Vec<(usize, usize, f64, Ray)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                result.push((
                    x,
                    y,
                    0.25,
                    self.ray_for_coordinate(x as f64 - 0.25, y as f64 - 0.25),
                ));
                result.push((
                    x,
                    y,
                    0.25,
                    self.ray_for_coordinate(x as f64 + 0.25, y as f64 - 0.25),
                ));
                result.push((
                    x,
                    y,
                    0.25,
                    self.ray_for_coordinate(x as f64 - 0.25, y as f64 + 0.25),
                ));
                result.push((
                    x,
                    y,
                    0.25,
                    self.ray_for_coordinate(x as f64 + 0.25, y as f64 + 0.25),
                ));
            }
        }
        return result;
    }
    fn get_differentials(&self, _r: &Ray) -> (Ray, Ray) {
        return (self.dxDifferential.clone(), self.dyDifferential.clone());
    }
}
