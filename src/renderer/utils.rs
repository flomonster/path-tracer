use super::Hit;
use super::Intersectable;
use super::Ray;
use crate::scene::internal::Model;
use crate::Scene;
use cgmath::*;
use rand::Rng;
use rand::rngs::StdRng;
use std::sync::Arc;

/// Return all the hits of a ray in a scene sorted by distance
pub fn ray_cast(scene: &Scene, ray: &Ray) -> Vec<(Hit, Arc<Model>)> {
    let mut res: Vec<_> = vec![];
    for model in scene.models.intersect(&ray.origin, &ray.direction) {
        for hit in model.intersect(ray) {
            res.push((hit, model.clone()));
        }
    }
    res.sort_by(|(hit1, _), (hit2, _)| hit1.get_dist().partial_cmp(&hit2.get_dist()).unwrap());
    res
}

pub fn russian_roulette(throughput: &mut Vector3<f32>, rand_gen: &mut StdRng) -> bool {
    // Randomly terminate a path with a probability inversely equal to the throughput
    let rr_proba = throughput.x.max(throughput.y).max(throughput.z);

    // Add the energy we 'lose' by randomly terminating paths
    *throughput *= 1. / rr_proba;

    rand_gen.gen::<f32>() > rr_proba
}

/// Compute reflection vector given incident and normal vectors
pub fn reflection(i: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    2. * i.dot(*n).max(0.) * n - i
}
