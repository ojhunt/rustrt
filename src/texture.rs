use colour::Colour;
use image::GenericImageView;
use scene::Scene;
use std::cell::*;
use std::ops::Deref;
use vectors::{Vec2d, Vec4d};

#[derive(Debug, Copy, Clone)]
pub struct TextureCoordinateIdx(pub usize);

impl TextureCoordinateIdx {
    pub fn get(&self, s: &Scene) -> Vec2d {
        let TextureCoordinateIdx(idx) = *self;
        return s.get_texture_coordinate(idx);
    }
}

trait Lerpable: Clone + Copy {
    fn scale(&self, f64) -> Self;
    fn add(&self, other: &Self) -> Self;
}

impl Lerpable for f64 {
    fn scale(&self, other: f64) -> Self {
        return *self * other;
    }
    fn add(&self, other: &Self) -> Self {
        return *self + other;
    }
}

impl Lerpable for Vec4d {
    fn scale(&self, other: f64) -> Self {
        return Vec4d::scale(*self, other);
    }
    fn add(&self, other: &Self) -> Self {
        return *self + *other;
    }
}

impl Lerpable for Colour {
    fn scale(&self, other: f64) -> Self {
        let Colour::RGB(r, g, b) = *self;
        return Colour::RGB(r * other, g * other, b * other);
    }
    fn add(&self, Colour::RGB(rr, rg, rb): &Self) -> Self {
        let Colour::RGB(r, g, b) = *self;
        return Colour::RGB(r + rr, g + rg, b + rb);
    }
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    width: usize,
    height: usize,
    data: Vec<Colour>,
    gradient_maps: RefCell<Option<(Vec<f64>, Vec<f64>)>>,
    foo: Cell<Option<f64>>,
}

impl Texture {
    pub fn new(name: &str, image: &image::DynamicImage) -> Texture {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let mut buffer: Vec<Colour> = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            buffer.push(Colour::RGB(0.0, 0.0, 0.0));
        }

        for (x, iy, pixel) in image.pixels() {
            let y = height - 1 - iy as usize;
            buffer[y * width + x as usize] = Colour::RGB(
                pixel[2] as f64 / 255.,
                pixel[1] as f64 / 255.,
                pixel[0] as f64 / 255.,
            );
        }

        return Texture {
            name: String::from(name),
            width: image.width() as usize,
            height: image.height() as usize,
            data: buffer,
            gradient_maps: RefCell::new(None),
            foo: Cell::new(None),
        };
    }

    fn get_raw_pixel<T: Lerpable>(&self, vec: &Vec<T>, x_: usize, y_: usize) -> T {
        let x = x_ % self.width;
        let y = y_ % self.height;
        let width = self.width;
        return vec[y * width + x];
    }

    fn lerp<T: Lerpable>(t: f64, l: &T, r: &T) -> T {
        return l.scale(1. - t).add(&r.scale(t));
    }

    fn get_pixel<T: Lerpable>(&self, vec: &Vec<T>, x: f64, y: f64) -> T {
        let xf = x.fract();
        let yf = y.fract();
        let xb = (x.floor() % self.width as f64) as usize;
        let yb = (y.floor() % self.height as f64) as usize;

        let tl = self.get_raw_pixel(vec, xb, yb);
        let tr = self.get_raw_pixel(vec, xb + 1, yb);

        let t = Self::lerp(xf, &tl, &tr);

        let bl = self.get_raw_pixel(vec, xb, yb + 1);
        let br = self.get_raw_pixel(vec, xb + 1, yb + 1);

        let b = Self::lerp(xf, &bl, &br);
        return Self::lerp(yf, &t, &b);
    }

    pub fn sample(&self, Vec2d(u, v): Vec2d) -> Colour {
        let x = u * self.width as f64;
        let y = v * self.height as f64;
        let xb = (x.floor() % self.width as f64) as usize;
        let yb = (y.floor() % self.height as f64) as usize;
        return self.get_raw_pixel(&self.data, xb, yb);
    }
    fn generate_gradient_maps(&self) -> (Vec<f64>, Vec<f64>) {
        let mut du: Vec<f64> = Vec::with_capacity(self.data.len());
        let mut dv: Vec<f64> = Vec::with_capacity(self.data.len());
        for _ in 0..(self.width * self.height) {
            du.push(0.0);
            dv.push(0.0);
        }
        for x in 0..self.width {
            for y in 0..self.height {
                let left = self.get_raw_pixel(&self.data, x - 1, y) * 2.
                    + self.get_raw_pixel(&self.data, x - 1, y - 1)
                    + self.get_raw_pixel(&self.data, x - 1, y + 1);
                let right = self.get_raw_pixel(&self.data, x + 1, y) * 2.
                    + self.get_raw_pixel(&self.data, x + 1, y - 1)
                    + self.get_raw_pixel(&self.data, x + 1, y + 1);
                let Colour::RGB(fu, _, _) = right - left;

                let top = self.get_raw_pixel(&self.data, x, y + 1) * 2.
                    + self.get_raw_pixel(&self.data, x - 1, y + 1)
                    + self.get_raw_pixel(&self.data, x + 1, y + 1);
                let bottom = self.get_raw_pixel(&self.data, x, y - 1) * 2.
                    + self.get_raw_pixel(&self.data, x - 1, y - 1)
                    + self.get_raw_pixel(&self.data, x + 1, y - 1);
                let Colour::RGB(fv, _, _) = top - bottom;
                du[y * self.width + x] = fu;
                dv[y * self.width + x] = fv;
            }
        }
        return (du, dv);
    }

    pub fn gradient(&self, Vec2d(u, v): Vec2d) -> (f64, f64) {
        let x = (u % 1.0) * self.width as f64;
        let y = (v % 1.0) * self.height as f64;

        if let Some((l, r)) = self.gradient_maps.borrow().deref() {
            let u = self.get_pixel(l, x, y);
            let v = self.get_pixel(r, x, y);
            return (u, v);
        }

        let (du, dv) = self.generate_gradient_maps();
        let result = (self.get_pixel(&du, x, y), self.get_pixel(&dv, x, y));
        self.gradient_maps.replace(Some((du, dv)));
        return result;
    }
}
