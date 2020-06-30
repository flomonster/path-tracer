use super::Vertex;
use crate::utils::{Intersectable, Ray};
use cgmath::InnerSpace;

pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let v0v1 = self.1.position - self.0.position;
        let v0v2 = self.2.position - self.0.position;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        // Check face culling
        if det < 0.0001 {
            return None;
        }

        let invdet = 1. / det;

        let tvec = ray.origin - self.0.position;
        let u = tvec.dot(pvec) * invdet;
        if u < 0. || u > 1. {
            return None;
        }

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * invdet;
        if v < 0. || v > 1. {
            return None;
        }
        Some(v0v2.dot(qvec) * invdet)
    }
}
