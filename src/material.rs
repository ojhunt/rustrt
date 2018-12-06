use colour::Colour;
use fragment::Fragment;
use ray::Ray;
use scene::Scene;
use std::fmt::Debug;
use vectors::Vec4d;

#[derive(Debug, Clone, Copy)]
pub enum Transparency {
    Opaque,
    Constant(f64),
    Halo(f64), // 1.0 - (N*v)(1.0-factor)
}

#[derive(Clone)]
pub struct MaterialCollisionInfo {
    pub ambient_colour: Colour,
    pub diffuse_colour: Colour,
    pub specular_colour: Colour,
    pub emissive_colour: Option<Colour>,
    pub transparent_colour: Option<Colour>,
    pub position: Vec4d,
    pub normal: Vec4d,
    pub secondaries: Vec<(Ray, Colour, f64)>,
}

pub trait Material: Debug {
    fn is_light(&self) -> bool;
    fn compute_surface_properties(&self, s: &Scene, f: &Fragment) -> MaterialCollisionInfo;
}
