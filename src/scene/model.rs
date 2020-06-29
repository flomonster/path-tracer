use crate::utils::{Material, Vertex};
use std::path::PathBuf;

pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

pub struct Model {
    pub triangles: Vec<Triangle>,
    pub material: Option<Material>,
}

impl Model {
    /// Create an empy model
    pub fn new() -> Self {
        Model {
            triangles: vec![],
            material: None,
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
            model.material = Some(Material::from((&materials[material_id], path)));
        }

        model
    }
}
