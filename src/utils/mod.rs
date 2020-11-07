mod hit;
mod ray;

pub use hit::Hit;
pub use ray::Intersectable;
pub use ray::Ray;

use cgmath::*;

pub fn reflection(I: &Vector3<f32>, N: &Vector3<f32>) -> Vector3<f32> {
    I - 2. * I.dot(N.clone()) * N
}
