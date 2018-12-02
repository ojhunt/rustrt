use colour::Colour;
use fragment::Fragment;
use image;
use material::MaterialCollisionInfo;
use material::{Material, Transparency};
use scene::Scene;
use scene::TextureIdx;
use vectors::Vec2d;
use vectors::Vec4d;

trait RawSurfaceValue: Clone + Clone + Copy {
    fn empty() -> Self;
}

impl RawSurfaceValue for Colour {
    fn empty() -> Self {
        return Colour::RGB(0.0, 0.0, 0.0);
    }
}

trait TextureSurfaceValue<Raw: RawSurfaceValue> {
    fn raw_for_fragment(&self, s: &Scene, f: &Fragment) -> Raw;
    fn gradient(&self, s: &Scene, point: Vec2d) -> (Vec4d, Vec4d);
}

impl TextureSurfaceValue<Colour> for TextureIdx {
    fn raw_for_fragment(&self, s: &Scene, f: &Fragment) -> Colour {
        return Colour::from(s.get_texture(*self).sample(f.uv));
    }
    fn gradient(&self, s: &Scene, uv: Vec2d) -> (Vec4d, Vec4d) {
        return s.get_texture(*self).gradient(uv);
    }
}

#[derive(Debug, Copy, Clone)]
enum WFSurfaceProperty<Raw: Copy + RawSurfaceValue, Texture: Copy + TextureSurfaceValue<Raw>> {
    None,
    Single(Raw),
    Texture(Texture),
    Complex(Raw, Texture),
}

impl<Raw: Copy + RawSurfaceValue, Texture: Copy + TextureSurfaceValue<Raw>>
    WFSurfaceProperty<Raw, Texture>
{
    pub fn new(raw: Option<Raw>, texture: Option<Texture>) -> WFSurfaceProperty<Raw, Texture> {
        match (raw, texture) {
            (Some(r), None) => WFSurfaceProperty::Single(r),
            (None, Some(t)) => WFSurfaceProperty::Texture(t),
            (Some(r), Some(t)) => WFSurfaceProperty::Complex(r, t),
            (None, None) => WFSurfaceProperty::None,
        }
    }
    pub fn raw_for_fragment(&self, scene: &Scene, fragment: &Fragment) -> Raw {
        return match self {
            WFSurfaceProperty::None => Raw::empty(),
            WFSurfaceProperty::Single(v) => *v,
            WFSurfaceProperty::Texture(t) => t.raw_for_fragment(scene, fragment),
            WFSurfaceProperty::Complex(_, t) => t.raw_for_fragment(scene, fragment),
            _ => panic!(),
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WFMaterial {
    // From http://paulbourke.net/dataformats/mtl/
    ambient_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ka
    diffuse_colour: WFSurfaceProperty<Colour, TextureIdx>, // Kd
    specular_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ks
    bump_map: Option<TextureIdx>,
    emissive_colour: WFSurfaceProperty<Colour, TextureIdx>, // Ke
    transparent_colour: Option<Colour>,                     // Tf
    transparency: Transparency, // d -- seriously who came up with these names?
    specular_exponent: Option<f64>, // Ns
    sharpness: Option<f64>,
    index_of_refraction: Option<f64>, // Ni
}

fn apply_bump_map(
    bump: Option<TextureIdx>,
    f: &Fragment,
    s: &Scene,
    m: &MaterialCollisionInfo,
) -> MaterialCollisionInfo {
    let mut new_info = m.clone();
    if bump.is_none() {
        //new_info.diffuse_colour = Colour::RGB(0.0, 1.0, 0.0);
        return new_info;
    }
    let map = s.get_texture(bump.unwrap());
    let (fu, fv) = {
        let (u, v) = map.gradient(f.uv);
        (u.x * 5.0, v.x * 5.0)
    };
    let n = m.normal;
    let ndpdv = n.cross(f.dpdv);
    let ndpdu = n.cross(f.dpdu);
    let mut perturbed_normal = n + (fu * ndpdv - fv * ndpdu);
    if perturbed_normal.dot(perturbed_normal) == 0.0 {
        perturbed_normal = n;
    } else {
        // new_info.diffuse_colour = Colour::RGB(0.4, 0.4, 0.4);

        // new_info.ambient_colour = new_info.diffuse_colour;
        // new_info.ambient_colour =
        //     Colour::from(perturbed_normal.normalize() * 0.5 + Vec4d::vector(0.5, 0.5, 0.5)); // Colour::RGB(fu.abs() * 4., fv.abs() * 4., 0.0);
        //                                                                                      // new_info.diffuse_colour = Colour::RGB(0.0, 0.0, 0.0);
    }

    // new_info.ambient_colour = Colour::from(
    //     (n + fu * ndpdv - fv * ndpdu).normalize() * 0.5 + Vec4d::vector(0.5, 0.5, 0.5),
    // );
    let mut temp = Vec4d::from(new_info.diffuse_colour);
    temp.y += (perturbed_normal.x - n.x).abs() * 500.;
    // new_info.diffuse_colour = Colour::from(temp);
    // new_info.diffuse_colour = new_info.ambient_colour; //Colour::RGB(0.5, f.uv.0.fract(), f.uv.1.fract());
    new_info.normal = perturbed_normal.normalize();
    return new_info;
}

impl Material for WFMaterial {
    fn compute_surface_properties(&self, s: &Scene, f: &Fragment) -> MaterialCollisionInfo {
        return apply_bump_map(
            self.bump_map,
            f,
            s,
            &MaterialCollisionInfo {
                ambient_colour: self.ambient_colour.raw_for_fragment(s, f),
                diffuse_colour: self.diffuse_colour.raw_for_fragment(s, f),
                specular_colour: self.specular_colour.raw_for_fragment(s, f),
                normal: f.normal,
                position: f.position,
                transparent_colour: None,
                secondaries: vec![],
            },
        );
    }
}

fn colour_from_slice(colour: Option<[f32; 3]>) -> Option<Colour> {
    match colour {
        None => None,
        Some([r, g, b]) => Some(Colour::RGB(r as f64, g as f64, b as f64)),
    }
}

fn load_texture<F: FnMut(&str) -> Option<TextureIdx>>(
    texture: &Option<String>,
    mut texture_loader: F,
) -> (Option<TextureIdx>, F) {
    return match texture {
        None => (None, texture_loader),
        Some(texture_name) => (texture_loader(texture_name), texture_loader),
    };
}

fn load_surface_colour<F: FnMut(&str) -> Option<TextureIdx>>(
    colour: Option<[f32; 3]>,
    texture: &Option<String>,
    mut texture_loader: F,
) -> (WFSurfaceProperty<Colour, TextureIdx>, F) {
    let real_colour = match colour {
        None => None,
        Some([r, g, b]) => Some(Colour::RGB(r as f64, g as f64, b as f64)),
    };
    let real_texture = match texture {
        None => None,
        Some(texture_name) => texture_loader(texture_name),
    };

    return (
        match (real_colour, real_texture) {
            (Some(colour), Some(texture)) => WFSurfaceProperty::Complex(colour, texture),
            (Some(colour), None) => WFSurfaceProperty::Single(colour),
            (None, Some(texture)) => WFSurfaceProperty::Texture(texture),
            _ => WFSurfaceProperty::None,
        },
        texture_loader,
    );
}

impl WFMaterial {
    pub fn new<F: FnMut(&str) -> Option<TextureIdx>>(
        mat: &obj::Material,
        texture_loader: F,
    ) -> WFMaterial {
        let opt_f32_to_f64 = |o: Option<f32>| {
            if let Some(v) = o {
                Some(v as f64)
            } else {
                None
            }
        };

        let (ambient, f) = load_surface_colour(mat.ka, &mat.map_ka, texture_loader);
        let (diffuse, f1) = load_surface_colour(mat.kd, &mat.map_kd, f);
        let (specular, f2) = load_surface_colour(mat.ks, &mat.map_ks, f1);
        let (emission, f3) = load_surface_colour(mat.ke, &mat.map_ke, f2);
        let (bump_map, _) = load_texture(&mat.map_bump, f3);

        WFMaterial {
            ambient_colour: ambient,
            diffuse_colour: diffuse,
            specular_colour: specular,
            emissive_colour: emission,
            bump_map: bump_map,
            transparent_colour: colour_from_slice(mat.tf),
            specular_exponent: opt_f32_to_f64(mat.ns),
            index_of_refraction: opt_f32_to_f64(mat.ni),
            transparency: if let Some(d) = mat.d {
                Transparency::Constant(d as f64)
            } else {
                Transparency::Opaque
            },
            sharpness: Some(1.),
        }
    }
}
