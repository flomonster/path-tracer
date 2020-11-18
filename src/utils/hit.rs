use cgmath::*;
use easy_gltf::model::Triangle;

#[derive(Debug, Clone, PartialEq)]
/// Describe an intersection between a ray and a triangle
pub struct Hit {
    pub triangle: Triangle,
    pub dist: f32,
    pub position: Vector3<f32>,

    /// Barycenter
    pub text_coords: Vector2<f32>,
    /// Normal vector of the triangle at the hit point
    pub normal: Vector3<f32>,
}

impl Hit {
    pub fn new(triangle: Triangle, dist: f32, position: Vector3<f32>, uv: &Vector2<f32>) -> Self {
        let normal = (1. - uv.x - uv.y) * triangle[0].normal
            + uv.x * triangle[1].normal
            + uv.y * triangle[2].normal;
        let text_coords = triangle[0].texture
            + uv.x * (triangle[1].texture - triangle[0].texture)
            + uv.y * (triangle[2].texture - triangle[0].texture);

        Self {
            triangle,
            dist,
            position,
            text_coords,
            normal,
        }
    }
}
