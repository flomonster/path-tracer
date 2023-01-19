use std::{ops::Deref, sync::Arc};

use cgmath::{ElementWise, Vector2, Vector3};
use image::{GrayImage, ImageBuffer, Pixel, RgbImage};

use crate::scene::isf;

use super::texture_bank::TextureBank;

#[derive(Clone, Debug)]
pub struct Material {
    /// Albedo
    pub albedo: Albedo,
    /// Emissive
    pub emissive: Emissive,
    /// Opacity
    pub opacity: Opacity,
    /// Metalness
    pub metalness: Metalness,
    /// Roughness
    pub roughness: Roughness,
    /// Index of refraction
    pub ior: f32,
    /// Normal texture
    pub normal_texture: Option<Arc<RgbImage>>,
}

#[derive(Clone, Debug)]
pub struct Albedo {
    factor: Vector3<f32>,
    texture: Option<Arc<RgbImage>>,
}

impl Albedo {
    fn load(albedo: isf::Albedo, texture_bank: &mut TextureBank) -> Self {
        Self {
            factor: albedo.factor.into(),
            texture: albedo.texture.map(|path| texture_bank.get_rgb(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Emissive {
    factor: Vector3<f32>,
    texture: Option<Arc<RgbImage>>,
}

impl Emissive {
    fn load(emissive: isf::Emissive, texture_bank: &mut TextureBank) -> Self {
        Self {
            factor: emissive.factor.into(),
            texture: emissive.texture.map(|path| texture_bank.get_rgb(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Opacity {
    factor: f32,
    texture: Option<Arc<GrayImage>>,
}

impl Opacity {
    fn load(opacity: isf::Opacity, texture_bank: &mut TextureBank) -> Self {
        Self {
            factor: opacity.factor,
            texture: opacity.texture.map(|path| texture_bank.get_gray(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metalness {
    factor: f32,
    texture: Option<Arc<GrayImage>>,
}

impl Metalness {
    fn load(metalness: isf::Metalness, texture_bank: &mut TextureBank) -> Self {
        Self {
            factor: metalness.factor,
            texture: metalness.texture.map(|path| texture_bank.get_gray(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Roughness {
    factor: f32,
    texture: Option<Arc<GrayImage>>,
}

impl Roughness {
    fn load(roughness: isf::Roughness, texture_bank: &mut TextureBank) -> Self {
        Self {
            factor: roughness.factor,
            texture: roughness.texture.map(|path| texture_bank.get_gray(path)),
        }
    }
}
impl Material {
    pub fn load(material: isf::Material, texture_bank: &mut TextureBank) -> Material {
        Self {
            albedo: Albedo::load(material.albedo, texture_bank),
            emissive: Emissive::load(material.emissive, texture_bank),
            opacity: Opacity::load(material.opacity, texture_bank),
            metalness: Metalness::load(material.metalness, texture_bank),
            roughness: Roughness::load(material.roughness, texture_bank),
            ior: material.ior,
            normal_texture: material.normal_texture.map(|t| texture_bank.get_rgb(t)),
        }
    }

    fn get_pixel<P, Container>(tex_coords: &Vector2<f32>, texture: &ImageBuffer<P, Container>) -> P
    where
        P: Pixel + 'static,
        P::Subpixel: 'static,
        Container: Deref<Target = [P::Subpixel]>,
    {
        let coords = tex_coords.mul_element_wise(Vector2::new(
            texture.width() as f32,
            texture.height() as f32,
        ));

        texture[(
            (coords.x as i64).rem_euclid(texture.width() as i64) as u32,
            (coords.y as i64).rem_euclid(texture.height() as i64) as u32,
        )]
    }

    pub fn get_albedo(&self, uv: &Vector2<f32>) -> Vector3<f32> {
        if let Some(texture) = &self.albedo.texture {
            let pixel = Self::get_pixel(uv, texture);
            // Convert sRGB to linear

            Vector3::new(
                (pixel[0] as f32 / 255.0).powf(2.2),
                (pixel[1] as f32 / 255.0).powf(2.2),
                (pixel[2] as f32 / 255.0).powf(2.2),
            )
            .mul_element_wise(self.albedo.factor)
        } else {
            self.albedo.factor
        }
    }

    pub fn get_simple_albedo(&self) -> Vector3<f32> {
        self.albedo.factor
    }

    pub fn get_metalness(&self, uv: &Vector2<f32>) -> f32 {
        if let Some(texture) = &self.metalness.texture {
            let pixel = Self::get_pixel(uv, texture);
            pixel[0] as f32 / 255. * self.metalness.factor
        } else {
            self.metalness.factor
        }
    }

    pub fn get_simple_metalness(&self) -> f32 {
        self.metalness.factor
    }

    pub fn get_roughness(&self, uv: &Vector2<f32>) -> f32 {
        if let Some(texture) = &self.roughness.texture {
            let pixel = Self::get_pixel(uv, texture);
            pixel[0] as f32 / 255. * self.roughness.factor
        } else {
            self.roughness.factor
        }
    }

    pub fn get_simple_roughness(&self) -> f32 {
        self.roughness.factor
    }

    pub fn get_normal(&self, uv: &Vector2<f32>) -> Option<Vector3<f32>> {
        self.normal_texture.as_ref().map(|texture| {
            let pixel = Self::get_pixel(uv, texture);
            Vector3::new(
                (pixel[0] as f32) / 127.5 - 1.,
                (pixel[1] as f32) / 127.5 - 1.,
                (pixel[2] as f32) / 127.5 - 1.,
            )
        })
    }

    pub fn get_emissive(&self, uv: &Vector2<f32>) -> Vector3<f32> {
        if let Some(texture) = &self.emissive.texture {
            let pixel = Self::get_pixel(uv, texture);
            Vector3::new(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            )
            .mul_element_wise(self.emissive.factor)
        } else {
            self.emissive.factor
        }
    }

    pub fn get_simple_emissive(&self) -> Vector3<f32> {
        self.emissive.factor
    }

    pub fn get_opacity(&self, uv: &Vector2<f32>) -> f32 {
        if let Some(texture) = &self.opacity.texture {
            let pixel = Self::get_pixel(uv, texture);
            pixel[0] as f32 / 255. * self.opacity.factor
        } else {
            self.opacity.factor
        }
    }

    pub fn get_simple_opacity(&self) -> f32 {
        self.opacity.factor
    }
}
