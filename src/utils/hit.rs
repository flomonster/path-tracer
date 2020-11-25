use crate::scene::model::Triangle;
use cgmath::*;

#[derive(Debug, Clone, PartialEq)]
/// Describe an intersection between a ray and a triangle
pub struct Hit {
    pub triangle: Triangle,
    pub dist: f32,
    pub position: Vector3<f32>,

    /// Texture coordinate
    pub tex_coords: Vector2<f32>,

    /// Normal vector of the triangle at the hit point
    pub normal: Vector3<f32>,
}

impl Hit {
    pub fn new(triangle: Triangle, dist: f32, position: Vector3<f32>, uv: &Vector2<f32>) -> Self {
        let normal = (1. - uv.x - uv.y) * triangle[0].normal
            + uv.x * triangle[1].normal
            + uv.y * triangle[2].normal;
        let tex_coords = triangle[0].tex_coords
            + uv.x * (triangle[1].tex_coords - triangle[0].tex_coords)
            + uv.y * (triangle[2].tex_coords - triangle[0].tex_coords);

        Self {
            triangle,
            dist,
            position,
            tex_coords,
            normal,
        }
    }
}
