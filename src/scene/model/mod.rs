mod triangle;

use easy_gltf::model::Triangle;
use easy_gltf::Material;
use std::sync::Arc;

use crate::utils::{Hit, Intersectable, Ray};

#[derive(Clone, Debug)]
pub struct Model {
    pub triangles: Vec<Triangle>,
    pub material: Arc<Material>,
}

impl Intersectable for Model {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let mut best = None;
        for t in self.triangles.iter() {
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

impl From<&easy_gltf::Model> for Model {
    fn from(eg_model: &easy_gltf::Model) -> Self {
        let model = Model {
            triangles: eg_model
                .triangles()
                .unwrap_or_else(|_| panic!("Model primitive isn't triangles")),
            material: eg_model.material().clone(),
        };
        model
    }
}
