use cgmath::*;
use easy_gltf::model::Triangle;

#[derive(Debug, Clone, PartialEq)]
/// Describe an intersection between a ray and a triangle
pub struct Hit {
    pub triangle: Triangle,
    pub dist: f32,
    pub position: Vector3<f32>,

    /// Barycenter
    pub uv: Vector2<f32>,
}

impl Hit {
    pub fn normal(&self) -> Vector3<f32> {
        (1. - self.uv.x - self.uv.y) * self.triangle[0].normal
            + self.uv.x * self.triangle[1].normal
            + self.uv.y * self.triangle[2].normal
    }
}
