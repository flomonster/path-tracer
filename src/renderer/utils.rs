use crate::scene::model::Model;
use crate::utils::{Hit, Intersectable, Ray};
use crate::Scene;
use cgmath::*;
use std::sync::Arc;

pub fn ray_cast(scene: &Scene, ray: &Ray) -> Option<(Hit, Arc<Model>)> {
    let mut best = None;
    for model in scene.models.intersect(&ray.origin, &ray.direction) {
        if let Some(hit) = model.intersect(ray) {
            best = match best {
                None => Some((hit, model)),
                Some((best_hit, _)) if best_hit.dist > hit.dist => Some((hit, model)),
                _ => best,
            }
        }
    }
    best
}
pub fn normal_tangent_to_world(normal: &Vector3<f32>, hit: &Hit) -> Vector3<f32> {
    let edge1 = hit.triangle[1].position - hit.triangle[0].position;
    let edge2 = hit.triangle[2].position - hit.triangle[0].position;
    let delta_uv1 = hit.triangle[1].tex_coords - hit.triangle[0].tex_coords;
    let delta_uv2 = hit.triangle[2].tex_coords - hit.triangle[0].tex_coords;

    let f = 1. / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
    let tangent = Vector3::new(
        f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
        f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
        f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
    )
    .normalize();

    let bitangent = hit.normal.cross(tangent);
    let tbn = Matrix3::from_cols(tangent, bitangent, hit.normal);
    (tbn * normal).normalize()
}
