use cgmath::*;
use derivative::Derivative;

use crate::scene::isf;

/// Using less fields than EasyVertex to optimize memory
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default)]
pub struct Vertex {
    #[derivative(Default(value = "Zero::zero()"))]
    pub position: Vector3<f32>,
    #[derivative(Default(value = "Zero::zero()"))]
    pub normal: Vector3<f32>,
    #[derivative(Default(value = "Zero::zero()"))]
    pub tex_coords: Vector2<f32>,
}

impl From<isf::Vertex> for Vertex {
    fn from(isf: isf::Vertex) -> Self {
        Self {
            position: isf.position.into(),
            normal: isf.normal.into(),
            tex_coords: isf.tex_coords.into(),
        }
    }
}
