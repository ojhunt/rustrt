
pub struct Collision {
    pub distance: f64,
    pub uv: (f64, f64)
}

impl Collision {
    pub fn new(distance: f64, uv: (f64, f64)) -> Collision {
        Collision{distance: distance, uv: uv}
    }
}