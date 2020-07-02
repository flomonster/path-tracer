use crate::scene::model::Triangle;
use cgmath::*;

#[derive(Debug, Clone, PartialEq)]
/// Describe an intersection between a ray and a triangle
pub struct Hit {
    pub triangle: Triangle,
    pub dist: f32,
    pub position: Vector3<f32>,

    /// Barycenter
    pub uv: Vector2<f32>,
}
