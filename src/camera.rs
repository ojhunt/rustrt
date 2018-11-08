use vec4d::Vec4d;

pub struct Camera {
    position: Vec4d,
    direction: Vec4d,
    up: Vec4d,
    fov: f64,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec4d::new(),
            direction: Vec4d::new(),
            up: Vec4d::new(),
            fov: 0.0
        }
    }
    pub fn goto(&mut self, position: Vec4d) {
        self.position = position;
    }
    pub fn lookAt(&mut self, location: Vec4d) {
        self.direction = (location - self.position).normalize();
    }
    
}
