#[derive(PartialEq, Debug)]
pub struct Collision {
    pub distance: f64,
    pub uv: (f64, f64),
    pub intersection_count: usize,
    pub node_count: usize,
}

impl Collision {
    pub fn new(distance: f64, uv: (f64, f64)) -> Collision {
        Collision {
            distance,
            uv,
            intersection_count: 0,
            node_count: 0,
        }
    }
}
