use cgmath::*;
use easy_gltf::model::Vertex as EasyVertex;

/// Using less fields than EasyVertex to optimize memory
#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

impl From<&EasyVertex> for Vertex {
    fn from(other: &EasyVertex) -> Self {
        Vertex {
            position: other.position,
            normal: other.normal,
            tex_coords: other.tex_coords,
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Zero::zero(),
            normal: Zero::zero(),
            tex_coords: Zero::zero(),
        }
    }
}
