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

impl From<Vec4d> for Colour {
    fn from(
        Vec4d {
            x: r,
            y: g,
            z: b,
            w: _a,
        }: Vec4d,
    ) -> Self {
        Colour::RGB(r, g, b)
    }
}
