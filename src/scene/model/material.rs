use cgmath::*;
use image::{open, GrayAlphaImage, RgbImage};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Material {
    /// Ambient color of the material
    pub ambient: Vector3<f32>,

    /// Diffuse color of the material
    pub diffuse: Vector3<f32>,

    /// Specular color of the material
    pub specular: Vector3<f32>,

    /// Shininess color of the material
    pub shininess: f32,

    /// Dissolve attribute is the alpha term for the material. Referred to as
    /// dissolve since that's what the MTL file format docs refer to it as
    pub dissolve: f32,

    /// Optical density also known as index of refraction. Called
    /// optical_density in the MTL specc. Takes on a value between 0.001 and
    /// 10.0. 1.0 means light does not bend as it passed through the object.
    pub optical_density: f32,

    /// Ambient texture image for the material
    ambient_texture: Option<RgbImage>,

    /// Diffuse texture image for the material
    diffuse_texture: Option<RgbImage>,

    /// Specular texture image for the material
    specular_texture: Option<RgbImage>,

    /// Normal map texture image for the material
    normal_texture: Option<RgbImage>,

    /// Shininess map texture image for the material
    shininess_texture: Option<RgbImage>,

    /// Dissolve (alpha) map texture image for the material
    dissolve_texture: Option<GrayAlphaImage>,

    /// Illumination properties of the object
    illumination: Option<u8>,
}

impl Material {
    /// Create default material (green)
    pub fn new() -> Self {
        Material {
            ambient: Vector3::new(0., 0., 0.),
            diffuse: Vector3::new(0., 1., 0.),
            specular: Vector3::new(0., 0., 0.),
            shininess: 0.,
            dissolve: 0.,
            optical_density: 1.,
            ambient_texture: None,
            diffuse_texture: None,
            specular_texture: None,
            normal_texture: None,
            shininess_texture: None,
            dissolve_texture: None,
            illumination: None,
        }
    }
}

impl From<(&tobj::Material, &PathBuf)> for Material {
    fn from(args: (&tobj::Material, &PathBuf)) -> Self {
        let (mat, path) = args;

        let ambient_texture = match open(path.join(&mat.ambient_texture)) {
            Ok(image) => Some(image.into_rgb()),
            _ => None,
        };

        let diffuse_texture = match open(path.join(&mat.diffuse_texture)) {
            Ok(image) => Some(image.into_rgb()),
            _ => None,
        };

        let specular_texture = match open(path.join(&mat.specular_texture)) {
            Ok(image) => Some(image.into_rgb()),
            _ => None,
        };

        let normal_texture = match open(path.join(&mat.normal_texture)) {
            Ok(image) => Some(image.into_rgb()),
            _ => None,
        };

        let shininess_texture = match open(path.join(&mat.shininess_texture)) {
            Ok(image) => Some(image.into_rgb()),
            _ => None,
        };

        let dissolve_texture = match open(path.join(&mat.dissolve_texture)) {
            Ok(image) => Some(image.into_luma_alpha()),
            _ => None,
        };

        Material {
            ambient: mat.ambient.into(),
            diffuse: mat.diffuse.into(),
            specular: mat.specular.into(),
            shininess: mat.shininess.into(),
            dissolve: mat.dissolve.into(),
            optical_density: mat.optical_density,
            ambient_texture,
            diffuse_texture,
            specular_texture,
            normal_texture,
            shininess_texture,
            dissolve_texture,
            illumination: mat.illumination_model,
        }
    }
}
