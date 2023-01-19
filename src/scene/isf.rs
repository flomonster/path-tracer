use cgmath::One;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Custom format of a scene
pub struct Scene {
    /// Models in the scene
    pub models: Vec<Model>,
    /// Camera of the scene
    pub camera: Camera,
    /// Lights in the scene
    pub lights: Vec<Light>,
    /// Background color of the scene
    pub background: [f32; 3],
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Custom format of a camera
pub struct Camera {
    pub transform: [[f32; 4]; 4],
    /// Fov in radians
    pub fov: f32,
    /// Far plane in meters
    pub zfar: f32,
    /// Near plane in meters
    pub znear: f32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
/// Custom format of a model
pub enum Model {
    Sphere {
        radius: f32,
        center: [f32; 3],
        material: Material,
    },
    Mesh {
        triangles: Vec<Triangle>,
        material: Material,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Custom format of a triangle
pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Custom format of a vertex
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
/// Custom format of a light
pub enum Light {
    Point {
        /// Position of the light
        position: [f32; 3],
        /// RGB color
        color: [f32; 3],
        /// Size of the light in meters
        size: f32,
    },
    Directional {
        /// Direction of the light
        direction: [f32; 3],
        /// RGB color
        color: [f32; 3],
    },
}

/// Custom format of a material
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    /// Albedo
    pub albedo: Albedo,
    /// Emissive
    #[serde(default)]
    pub emissive: Emissive,
    /// Opacity
    #[serde(default)]
    pub opacity: Opacity,
    /// Metalness
    #[serde(default)]
    pub metalness: Metalness,
    /// Roughness
    #[serde(default)]
    pub roughness: Roughness,
    /// Index of refraction
    #[serde(default = "One::one")]
    pub ior: f32,
    /// Normal texture
    pub normal_texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Albedo {
    #[serde(default = "one")]
    pub factor: [f32; 3],
    pub texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Emissive {
    #[serde(default = "one")]
    pub factor: [f32; 3],
    pub texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Opacity {
    #[serde(default = "One::one")]
    #[derivative(Default(value = "1.0"))]
    pub factor: f32,
    pub texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Metalness {
    #[serde(default = "One::one")]
    pub factor: f32,
    pub texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Roughness {
    #[serde(default = "One::one")]
    #[derivative(Default(value = "1.0"))]
    pub factor: f32,
    pub texture: Option<String>,
}

fn one() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}
