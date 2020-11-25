mod triangle;

use crate::utils::*;
use easy_gltf::Material;
use kdtree_ray::{BoundingBox, KDtree, AABB};
use std::sync::Arc;
pub use triangle::Triangle;

#[derive(Clone, Debug)]
pub struct Model {
    triangles: KDtree<Triangle>,
    pub material: Arc<Material>,
}

impl Intersectable<Option<Hit>> for Model {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let mut best = None;
        for t in self.triangles.intersect(&ray.origin, &ray.direction).iter() {
            if let Some(hit) = t.intersect(ray) {
                best = match best {
                    None => Some(hit),
                    Some(best_hit) if best_hit.dist > hit.dist => Some(hit),
                    _ => best,
                }
            }
        }
        best
    }
}

impl BoundingBox for Model {
    fn bounding_box(&self) -> AABB {
        self.triangles.bounding_box()
    }
}

impl From<&easy_gltf::Model> for Model {
    fn from(eg_model: &easy_gltf::Model) -> Self {
        let kdtree = KDtree::new(
            eg_model
                .triangles()
                .unwrap_or_else(|_| panic!("Model primitive isn't triangles"))
                .iter()
                .map(|t| Triangle::from(t.clone()))
                .collect(),
        );
        Model {
            triangles: kdtree,
            material: eg_model.material().clone(),
        }
    }
}
