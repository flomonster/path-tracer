use crate::utils::vertex::Vertex;

pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

pub struct Model {
    pub triangles: Vec<Triangle>,
    //textures: Vec<>; TODO: Add textures
}

impl Model {
    pub fn new() -> Self {
        Model { triangles: vec![] }
    }
}

impl From<&tobj::Model> for Model {
    fn from(obj: &tobj::Model) -> Self {
        let obj = &obj.mesh;
        let mut model = Model::new();

        for i in (0..obj.indices.len()).step_by(3) {
            let mut vertices: Vec<Vertex> = Vec::new();
            for j in i..(i+3) {
                vertices.push(
                    Vertex::new(obj.positions[obj.indices[j] as usize * 3],
                                obj.positions[obj.indices[j] as usize * 3 + 1],
                                obj.positions[obj.indices[j] as usize * 3 + 2],
                                obj.normals[obj.indices[j] as usize * 3],
                                obj.normals[obj.indices[j] as usize * 3 + 1],
                                obj.normals[obj.indices[j] as usize * 3 + 2],
                                obj.texcoords[obj.indices[j] as usize * 2],
                                obj.texcoords[obj.indices[j] as usize * 2 + 1])
                )
            }
            model.triangles.push(Triangle(
                vertices[0].clone(),
                vertices[1].clone(),
                vertices[2].clone()
            ));
        }
        model
    }
}