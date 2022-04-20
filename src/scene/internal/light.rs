use cgmath::Vector3;

use crate::scene::isf;

#[derive(Debug, Clone)]
pub enum Light {
    Point {
        position: Vector3<f32>,
        color: Vector3<f32>,
        size: f32,
    },
    Directional {
        direction: Vector3<f32>,
        color: Vector3<f32>,
    },
}

impl From<isf::Light> for Light {
    fn from(l: isf::Light) -> Self {
        match l {
            isf::Light::Point {
                position,
                color,
                size,
            } => Light::Point {
                position: position.into(),
                color: color.into(),
                size,
            },
            isf::Light::Directional { direction, color } => Light::Directional {
                direction: direction.into(),
                color: color.into(),
            },
        }
    }
}
