use crate::scene::internal::Model;
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
                Some((best_hit, _)) if best_hit.get_dist() > hit.get_dist() => Some((hit, model)),
                _ => best,
            }
        }
    }
    best
}

pub fn russian_roulette(throughput: &mut Vector3<f32>) -> bool {
    // Randomly terminate a path with a probability inversely equal to the throughput
    let rr_proba = throughput.x.max(throughput.y).max(throughput.z);

    // Add the energy we 'lose' by randomly terminating paths
    *throughput *= 1. / rr_proba;

    rand::random::<f32>() > rr_proba
}
