use vectors::Vec4d;

#[derive(Debug, Copy, Clone)]
pub enum Colour {
    RGB(f64, f64, f64),
}

impl From<Colour> for Vec4d {
    fn from(Colour::RGB(x, y, z): Colour) -> Self {
        Vec4d::vector(x, y, z)
    }
}
