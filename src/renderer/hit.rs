use super::material_sample::MaterialSample;
use crate::scene::internal::{Material, Model, Triangle};
use cgmath::*;

#[derive(Debug, Clone, PartialEq)]
/// Describe an intersection between a ray and a triangle
pub enum Hit {
    Triangle {
        /// Distance from the ray origin to the intersection point
        dist: f32,

        /// Position of the intersection
        position: Vector3<f32>,

        /// Normal vector of the triangle at the hit point
        normal: Vector3<f32>,

        // Tangent vector of the triangle at the hit point
        tangent: Vector3<f32>,

        /// Texture coordinate
        tex_coords: Vector2<f32>,
    },
    Sphere {
        /// Distance from the ray origin to the intersection point
        dist: f32,

        /// Position of the intersection
        position: Vector3<f32>,

        /// Normal vector of the triangle at the hit point
        normal: Vector3<f32>,
    },
}

impl Hit {
    pub fn get_dist(&self) -> f32 {
        match self {
            Hit::Triangle { dist, .. } => *dist,
            Hit::Sphere { dist, .. } => *dist,
        }
    }

    pub fn get_geometric_normal(&self) -> Vector3<f32> {
        match self {
            Hit::Triangle { normal, .. } => *normal,
            Hit::Sphere { normal, .. } => *normal,
        }
    }

    pub fn get_normal(&self, material: &Material) -> Vector3<f32> {
        match self {
            Hit::Triangle {
                normal: hit_normal,
                tangent: hit_tangent,
                tex_coords,
                ..
            } => {
                if let Some(normal_map) = material.get_normal(tex_coords) {
                    // Compute the normal vector from the texture normal
                    let bitangent = hit_normal.cross(*hit_tangent);
                    let tbn = Matrix3::from_cols(*hit_tangent, bitangent, *hit_normal);
                    (tbn * normal_map).normalize()
                } else {
                    *hit_normal
                }
            }
            Hit::Sphere { normal, .. } => *normal,
        }
    }

    pub fn get_material_sample(&self, model: &Model) -> MaterialSample {
        match self {
            Hit::Sphere { .. } => MaterialSample::simple(model.get_material()),
            Hit::Triangle { tex_coords, .. } => {
                MaterialSample::new(model.get_material(), tex_coords)
            }
        }
    }

    pub fn get_position(&self) -> Vector3<f32> {
        match self {
            Hit::Triangle { position, .. } => *position,
            Hit::Sphere { position, .. } => *position,
        }
    }

    pub fn new_triangle(
        triangle: Triangle,
        dist: f32,
        position: Vector3<f32>,
        uv: &Vector2<f32>,
    ) -> Self {
        // Interpolate normal from vertices
        let normal = (1. - uv.x - uv.y) * triangle[0].normal
            + uv.x * triangle[1].normal
            + uv.y * triangle[2].normal;
        let tex_coords = triangle[0].tex_coords
            + uv.x * (triangle[1].tex_coords - triangle[0].tex_coords)
            + uv.y * (triangle[2].tex_coords - triangle[0].tex_coords);

        // Interpolate tangent from vertices
        let edge1 = triangle[1].position - triangle[0].position;
        let edge2 = triangle[2].position - triangle[0].position;
        let delta_uv1 = triangle[1].tex_coords - triangle[0].tex_coords;
        let delta_uv2 = triangle[2].tex_coords - triangle[0].tex_coords;

        let f = 1. / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
        let tangent = Vector3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        )
        .normalize();

        Self::Triangle {
            dist,
            position,
            normal,
            tex_coords,
            tangent,
        }
    }
}
