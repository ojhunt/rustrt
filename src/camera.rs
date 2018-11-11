use ray::Ray;
use vec4d::Vec4d;

pub struct Camera {
    position: Vec4d,
    direction: Vec4d,
    fov: f64,
}

impl Camera {
    pub fn new(position: Vec4d, target: Vec4d, fov: f64) -> Camera {
        Camera {
            position: position,
            direction: (target - position).normalize(),
            fov: fov,
        }
    }
    pub fn get_rays(&self, width: usize, height: usize) -> Vec<Ray> {
        let mut result: Vec<Ray> = Vec::new();
        let right = self.direction.cross(Vec4d::point(0., 1., 0.)).normalize();
        let up = right.cross(self.direction).normalize();

        let half_width = (self.fov.to_radians() / 2.).tan();
        let aspect_ratio = height as f64 / width as f64;
        let half_height = aspect_ratio * half_width;
        let view_origin = (self.position + self.direction) + up * half_height - right * half_width;

        let x_delta = (right * 2. * half_width) * (1. / width as f64);
        let y_delta = (up * 2. * half_height) * (1. / height as f64);

        for y in 0..height {
            for x in 0..width {
                let view_target = view_origin + (x_delta * x as f64) - (y_delta * y as f64);

                result.push(Ray::new(
                    self.position,
                    (view_target - self.position).normalize(),
                ));
            }
        }
        return result;
    }
}
