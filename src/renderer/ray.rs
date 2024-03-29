use cgmath::*;

#[derive(Debug)]
pub struct Ray {
    /// Origin of the ray
    pub origin: Vector3<f32>,

    /// Direction of the ray
    pub direction: Vector3<f32>,
}

impl Ray {
    /// Create a Ray given origin and direction
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        Ray { origin, direction }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            origin: Vector3::zero(),
            direction: Vector3::zero(),
        }
    }
}

pub trait Intersectable<P> {
    fn intersect(&self, ray: &Ray) -> P;
}
