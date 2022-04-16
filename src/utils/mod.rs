mod hit;
#[allow(dead_code)]
pub mod logger;
mod ray;

pub use hit::Hit;
pub use ray::*;

use cgmath::*;

/// Compute reflection vector given incident and normal vectors
pub fn reflection(i: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    2. * i.dot(*n).max(0.) * n - i
}
