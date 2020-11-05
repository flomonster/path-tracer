mod material;
mod triangle;
mod vertex;

pub use material::Material;
pub use triangle::Triangle;
pub use vertex::Vertex;

use crate::utils::{Hit, Intersectable, Ray};
use std::path::PathBuf;

pub struct Model {
    pub triangles: Vec<Triangle>,
    pub material: Material,
}

impl Model {
    /// Create an empy model
    pub fn new() -> Self {
        Model {
            triangles: vec![],
            material: Material::new(),
        }
    }

    /// Load a model from tobj Model and Material
    /// Need base path of the .obj to retrieve textures
    pub fn load(obj: &tobj::Model, materials: &Vec<tobj::Material>, path: &PathBuf) -> Self {
        let obj = &obj.mesh;
        let mut model = Model::new();

        for i in (0..obj.indices.len()).step_by(3) {
            let mut vertices: Vec<Vertex> = Vec::new();
            for j in i..(i + 3) {
                vertices.push(Vertex::new(
                    obj.positions[obj.indices[j] as usize * 3],
                    obj.positions[obj.indices[j] as usize * 3 + 1],
                    obj.positions[obj.indices[j] as usize * 3 + 2],
                    obj.normals[obj.indices[j] as usize * 3],
                    obj.normals[obj.indices[j] as usize * 3 + 1],
                    obj.normals[obj.indices[j] as usize * 3 + 2],
                    obj.texcoords[obj.indices[j] as usize * 2],
                    obj.texcoords[obj.indices[j] as usize * 2 + 1],
                ))
            }
            model.triangles.push(Triangle(
                vertices[0].clone(),
                vertices[1].clone(),
                vertices[2].clone(),
            ));
        }

        if let Some(material_id) = obj.material_id {
            model.material = Material::from((&materials[material_id], path));
        }

        model
    }
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
