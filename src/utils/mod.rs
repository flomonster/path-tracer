mod hit;
mod ray;

pub use hit::Hit;
pub use ray::Intersectable;
pub use ray::Ray;

use cgmath::*;

/// Compute reflection vector given incident and normal vectors
pub fn _reflection(i: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    i - 2. * i.dot(n.clone()) * n
}
