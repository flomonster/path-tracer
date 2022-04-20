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
pub enum Albedo {
    Value(Vector3<f32>),
    Texture(Arc<RgbImage>),
}

impl Albedo {
    fn load(albedo: isf::Albedo, texture_bank: &mut TextureBank) -> Self {
        match albedo {
            isf::Albedo::Value(value) => Self::Value(value.into()),
            isf::Albedo::Texture(path) => Self::Texture(texture_bank.get_rgb(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Emissive {
    Value(Vector3<f32>),
    Texture(Arc<RgbImage>),
}

impl Emissive {
    fn load(emissive: isf::Emissive, texture_bank: &mut TextureBank) -> Self {
        match emissive {
            isf::Emissive::Value(value) => Self::Value(value.into()),
            isf::Emissive::Texture(path) => Self::Texture(texture_bank.get_rgb(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Opacity {
    Value(f32),
    Texture(Arc<GrayImage>),
}

impl Opacity {
    fn load(emissive: isf::Opacity, texture_bank: &mut TextureBank) -> Self {
        match emissive {
            isf::Opacity::Value(value) => Self::Value(value),
            isf::Opacity::Texture(path) => Self::Texture(texture_bank.get_gray(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Metalness {
    Value(f32),
    Texture(Arc<GrayImage>),
}

impl Metalness {
    fn load(metalness: isf::Metalness, texture_bank: &mut TextureBank) -> Self {
        match metalness {
            isf::Metalness::Value(value) => Self::Value(value),
            isf::Metalness::Texture(path) => Self::Texture(texture_bank.get_gray(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Roughness {
    Value(f32),
    Texture(Arc<GrayImage>),
}

impl Roughness {
    fn load(roughness: isf::Roughness, texture_bank: &mut TextureBank) -> Self {
        match roughness {
            isf::Roughness::Value(value) => Self::Value(value),
            isf::Roughness::Texture(path) => Self::Texture(texture_bank.get_gray(path)),
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
        match &self.albedo {
            Albedo::Value(value) => *value,
            Albedo::Texture(texture) => {
                let pixel = Self::get_pixel(uv, texture);
                // Convert sRGB to linear
                Vector3::new(
                    (pixel[0] as f32 / 255.0).powf(2.2),
                    (pixel[1] as f32 / 255.0).powf(2.2),
                    (pixel[2] as f32 / 255.0).powf(2.2),
                )
            }
        }
    }

    pub fn get_simple_albedo(&self) -> Vector3<f32> {
        match &self.albedo {
            Albedo::Value(value) => *value,
            _ => panic!("Albedo texture is not supported"),
        }
    }

    pub fn get_metalness(&self, uv: &Vector2<f32>) -> f32 {
        match &self.metalness {
            Metalness::Value(value) => *value,
            Metalness::Texture(texture) => {
                let pixel = Self::get_pixel(uv, texture);
                pixel[0] as f32 / 255.
            }
        }
    }

    pub fn get_simple_metalness(&self) -> f32 {
        match &self.metalness {
            Metalness::Value(value) => *value,
            _ => panic!("Metalness texture not supported"),
        }
    }

    pub fn get_roughness(&self, uv: &Vector2<f32>) -> f32 {
        match &self.roughness {
            Roughness::Value(value) => *value,
            Roughness::Texture(texture) => {
                let pixel = Self::get_pixel(uv, texture);
                pixel[0] as f32 / 255.
            }
        }
    }

    pub fn get_simple_roughness(&self) -> f32 {
        match &self.roughness {
            Roughness::Value(value) => *value,
            _ => panic!("Roughness texture not supported"),
        }
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
        match &self.emissive {
            Emissive::Value(value) => *value,
            Emissive::Texture(texture) => {
                let pixel = Self::get_pixel(uv, texture);
                Vector3::new(
                    pixel[0] as f32 / 255.0,
                    pixel[1] as f32 / 255.0,
                    pixel[2] as f32 / 255.0,
                )
            }
        }
    }

    pub fn get_simple_emissive(&self) -> Vector3<f32> {
        match &self.emissive {
            Emissive::Value(value) => *value,
            _ => panic!("Emissive texture not supported"),
        }
    }

    pub fn get_opacity(&self, uv: &Vector2<f32>) -> f32 {
        match &self.opacity {
            Opacity::Value(value) => *value,
            Opacity::Texture(texture) => {
                let pixel = Self::get_pixel(uv, texture);
                pixel[0] as f32 / 255.
            }
        }
    }

    pub fn get_simple_opacity(&self) -> f32 {
        match &self.opacity {
            Opacity::Value(value) => *value,
            _ => panic!("Opacity texture not supported"),
        }
    }
}
