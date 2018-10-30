use vec4d::Vec4d;
use ray::Ray;

#[derive(Debug)]
pub struct Triangle {
    pub origin: Vec4d,
    pub edges: [Vec4d; 2]
}

impl Triangle {
    pub fn new(v0: Vec4d, v1: Vec4d, v2: Vec4d) -> Triangle {
        let edge0 = v1 - v0;
        let edge1 = v2 - v0;
        Triangle {
            origin: v0,
            edges: [edge0, edge1]
        }
    }

    pub fn intersect(&self, ray: Ray) -> f64 {
        let h = ray.direction.cross(self.edges[0]);
        let a = self.edge1.dot(h);
        
        return 0.0;
    }
}