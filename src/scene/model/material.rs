use crate::utils::Hit;
use cgmath::*;
use image::{open, GrayAlphaImage, ImageBuffer, Pixel, RgbImage};
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Material {
    /// Ambient color of the material
    ambient: Vector3<f32>,

    /// Diffuse color of the material
    diffuse: Vector3<f32>,

    /// Specular color of the material
    specular: Vector3<f32>,

    /// Shininess color of the material
    shininess: f32,

    /// Dissolve attribute is the alpha term for the material. Referred to as
    /// dissolve since that's what the MTL file format docs refer to it as
    dissolve: f32,

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
    shininess_texture: Option<GrayAlphaImage>,

    /// Dissolve (alpha) map texture image for the material
    dissolve_texture: Option<GrayAlphaImage>,

    /// Illumination properties of the object
    pub illumination: Option<u8>,
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
            Ok(image) => Some(image.into_luma_alpha()),
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

impl Material {
    /// Return pixel coords given an image and a hit
    fn compute_texture_coords<P, C>(img: &ImageBuffer<P, C>, hit: &Hit) -> Vector2<u32>
    where
        P: Pixel + 'static,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
    {
        let coords = hit.triangle.0.texture
            + hit.uv.x * (hit.triangle.1.texture - hit.triangle.0.texture)
            + hit.uv.y * (hit.triangle.2.texture - hit.triangle.0.texture);
        let coords = coords.mul_element_wise(Vector2::new(img.width() as f32, img.height() as f32));
        Vector2::new(coords.x as u32, img.height() - coords.y as u32)
    }

    fn get_color(img: &RgbImage, coords: Vector2<u32>) -> Vector3<f32> {
        let px = img[(coords.x, coords.y)];
        Vector3::new(
            px[0] as f32 / 255.,
            px[1] as f32 / 255.,
            px[2] as f32 / 255.,
        )
    }

    fn get_alpha(img: &GrayAlphaImage, coords: Vector2<u32>) -> f32 {
        let px = img[(coords.x, coords.y)];
        (px[1] as f32) / 255.
    }

    /// Return the appropriate diffuse color
    pub fn get_diffuse(&self, hit: &Hit) -> Vector3<f32> {
        if let Some(img) = &self.diffuse_texture {
            let coords = Self::compute_texture_coords(img, hit);
            Self::get_color(img, coords)
        } else {
            self.diffuse
        }
    }

    /// Return the appropriate specular color
    pub fn get_specular(&self, hit: &Hit) -> Vector3<f32> {
        if let Some(img) = &self.specular_texture {
            let coords = Self::compute_texture_coords(img, hit);
            Self::get_color(img, coords)
        } else {
            self.specular
        }
    }

    /// Return the appropriate ambient color
    pub fn get_ambient(&self, hit: &Hit) -> Vector3<f32> {
        if let Some(img) = &self.ambient_texture {
            let coords = Self::compute_texture_coords(img, hit);
            Self::get_color(img, coords)
        } else {
            self.ambient
        }
    }

    /// Return the appropriate shininess color
    pub fn get_shininess(&self, hit: &Hit) -> f32 {
        if let Some(img) = &self.shininess_texture {
            let coords = Self::compute_texture_coords(img, hit);
            Self::get_alpha(img, coords)
        } else {
            self.shininess
        }
    }

    /// Return the appropriate dissolve color
    pub fn get_dissolve(&self, hit: &Hit) -> f32 {
        if let Some(img) = &self.dissolve_texture {
            let coords = Self::compute_texture_coords(img, hit);
            Self::get_alpha(img, coords)
        } else {
            self.dissolve
        }
    }
}
