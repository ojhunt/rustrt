use camera::Camera;
use collision::Collision;
use compound_object::CompoundObject;
use image::*;
use intersectable::Intersectable;
use ray::Ray;
use vec4d::Vec4d;

#[derive(Debug)]
pub struct Scene {
    _scene: CompoundObject,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            _scene: CompoundObject::new(),
        }
    }
    pub fn add_object(&mut self, object: Box<Intersectable>) {
        self._scene.add_object(object)
    }

    pub fn intersect(&self, ray: Ray) -> Option<Collision> {
        return self._scene.intersect(ray, 0.0, std::f64::INFINITY);
    }

    pub fn finalize(&mut self) {
        self._scene.finalize();
    }

    pub fn render(&self, camera: &Camera, size: usize) -> DynamicImage {
        let mut result = image::RgbImage::new(size as u32, size as u32);
        let mut buffer = vec![(0 as f64, 0 as f64, 0 as f64); size * size];

        let mut min_depth = std::f64::INFINITY;
        let mut max_depth = -std::f64::INFINITY;
        let mut min_nodecount = 0;
        let mut max_nodecount = 0;
        let mut min_intersectount = 0;
        let mut max_intersectcount = 0;
        let rays = camera.get_rays(size, size);
        for x in 0..size {
            for y in 0..size {
                let ray = rays[x + size * y];
                match self.intersect(ray) {
                    None => continue,
                    Some(Collision {
                        distance: d,
                        uv: _,
                        intersection_count,
                        node_count,
                    }) => {
                        max_depth = max_depth.max(d);
                        min_depth = min_depth.min(d);
                        max_nodecount = max_nodecount.max(node_count);
                        min_nodecount = min_nodecount.min(node_count);
                        max_intersectcount = max_intersectcount.max(intersection_count);
                        min_intersectount = min_intersectount.min(intersection_count);
                        buffer[x + y * size] = (d, intersection_count as f64, node_count as f64);
                    }
                }
            }
        }

        println!(
            "Minimum intersections: {}, max: {}",
            min_intersectount, max_intersectcount
        );
        for (x, y, _pixel) in result.enumerate_pixels_mut() {
            let (d, ic, nc) = buffer[x as usize + y as usize * size];
            let scaled_depth = (255. * (1. - (d - min_depth) / (max_depth - min_depth)))
                .max(0.)
                .min(255.) as u8;
            let scaled_intersection_count = (255. * (ic - min_intersectount as f64) as f64
                / (max_intersectcount - min_intersectount) as f64)
                .max(0.)
                .min(255.) as u8;
            let scaled_node_count = ((nc - min_nodecount as f64) as f64
                / (max_nodecount - min_nodecount) as f64)
                .min(0.)
                .max(255.) as u8;
            *_pixel = image::Rgb([
                scaled_depth * 1,
                scaled_intersection_count * 0,
                scaled_node_count * 0,
            ]);
        }

        return ImageRgb8(result);
    }
}
