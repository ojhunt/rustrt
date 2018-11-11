use ray::Ray;
use vec4d::Vec4d;

pub struct Camera {
    position: Vec4d,
    target: Vec4d,
    up: Vec4d,
    fov: f64,
    sensor_size: f64,
}

impl Camera {
    pub fn new(position: Vec4d, target: Vec4d, fov: f64) -> Camera {
        Camera {
            position: position,
            target: target,
            up: Vec4d::new(),
            fov: fov,
            sensor_size: 3.,
        }
    }
    pub fn get_rays(&self, width: usize, height: usize) -> Vec<Ray> {
        let mut result: Vec<Ray> = Vec::new();
        let viewDirection = (self.target - self.position).normalize();
        let mut U = viewDirection.cross(Vec4d::point(0., 1., 0.));
        let mut V = U.cross(viewDirection);
        U = U.normalize();
        V = V.normalize();
        let viewPlaneHalfWidth = (self.fov.to_radians() / 2.).tan();
        let aspectRatio = height as f64 / width as f64;
        let viewPlaneHalfHeight = aspectRatio * viewPlaneHalfWidth;
        let viewPlaneTopLeft =
            (self.position + viewDirection) + V * viewPlaneHalfHeight - U * viewPlaneHalfWidth;
        let halfWidth = width as f64 / 2.;
        let halfHeight = height as f64 / 2.;

        let xIncVector = (U * 2. * viewPlaneHalfWidth) * (1. / width as f64);
        let yIncVector = (V * 2. * viewPlaneHalfHeight) * (1. / height as f64);

        for y in 0..height {
            for x in 0..width {
                let viewPlanePoint =
                    viewPlaneTopLeft + (xIncVector * x as f64) - (yIncVector * y as f64);
                let pixelDirection = (viewPlanePoint - self.position).normalize();

                result.push(Ray::new(self.position, pixelDirection));
            }
        }
        return result;
    }
}
