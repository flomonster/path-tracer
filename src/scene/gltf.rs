use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

use image::{DynamicImage, GrayImage, Luma, RgbImage, RgbaImage};
use serde_json::to_writer;

use crate::scene::isf::Roughness;

use super::isf::{
    Albedo, Camera, Emissive, Light, Material, Metalness, Model, Opacity, Scene, Triangle, Vertex,
};
use std::error::Error;

#[derive(Debug, Default)]
struct ReverseTextureBank {
    rgb_textures: HashMap<Arc<RgbImage>, String>,
    albedo_textures: HashMap<Arc<RgbaImage>, String>,
    alpha_textures: HashMap<Arc<RgbaImage>, String>,
    gray_textures: HashMap<Arc<GrayImage>, String>,
}

impl ReverseTextureBank {
    pub fn save(&self, dir: &Path) {
        for (image, path) in self.rgb_textures.iter() {
            image.save(dir.join(path)).unwrap();
        }
        for (image, path) in self.gray_textures.iter() {
            image.save(dir.join(path)).unwrap();
        }
        for (image, path) in self.albedo_textures.iter() {
            let image: DynamicImage = ((**image).clone()).into();
            image.into_rgb8().save(dir.join(path)).unwrap();
        }
        for (image, path) in self.alpha_textures.iter() {
            let image: DynamicImage = GrayImage::from_fn(image.width(), image.height(), |x, y| {
                Luma([image.get_pixel(x, y)[3]])
            })
            .into();
            image.save(dir.join(path)).unwrap();
        }
    }

    pub fn get_rgb_path(&mut self, image: Arc<RgbImage>) -> String {
        let len = self.rgb_textures.len();
        self.rgb_textures
            .entry(image)
            .or_insert(format!("vec_tex_{len}.png"))
            .clone()
    }

    pub fn get_gray_path(&mut self, image: Arc<GrayImage>) -> String {
        let len = self.gray_textures.len();
        self.gray_textures
            .entry(image)
            .or_insert(format!("gray_tex_{len}.png"))
            .clone()
    }

    pub fn get_alpha_path(&mut self, image: Arc<RgbaImage>) -> String {
        let len = self.alpha_textures.len();
        self.alpha_textures
            .entry(image)
            .or_insert(format!("alpha_tex_{len}.png"))
            .clone()
    }

    pub fn get_albedo_path(&mut self, image: Arc<RgbaImage>) -> String {
        let len = self.albedo_textures.len();
        self.albedo_textures
            .entry(image)
            .or_insert(format!("albedo_tex_{len}.png"))
            .clone()
    }
}
fn convert_material(
    material: Arc<easy_gltf::Material>,
    reverse_texture: &mut ReverseTextureBank,
) -> Material {
    Material {
        albedo: Albedo {
            factor: material.pbr.base_color_factor.truncate().into(),
            texture: material
                .pbr
                .base_color_texture
                .clone()
                .map(|tex| reverse_texture.get_albedo_path(tex)),
        },
        emissive: Emissive {
            factor: material.emissive.factor.into(),
            texture: material
                .emissive
                .texture
                .clone()
                .map(|tex| reverse_texture.get_rgb_path(tex)),
        },
        opacity: Opacity {
            factor: material.pbr.base_color_factor[3],
            texture: material
                .pbr
                .base_color_texture
                .clone()
                .map(|tex| reverse_texture.get_alpha_path(tex)),
        },
        metalness: Metalness {
            factor: material.pbr.metallic_factor,
            texture: material
                .pbr
                .metallic_texture
                .clone()
                .map(|tex| reverse_texture.get_gray_path(tex)),
        },
        roughness: Roughness {
            factor: material.pbr.roughness_factor,
            texture: material
                .pbr
                .roughness_texture
                .clone()
                .map(|tex| reverse_texture.get_gray_path(tex)),
        },
        ior: 1.0,
        normal_texture: material
            .normal
            .clone()
            .map(|texture| reverse_texture.get_rgb_path(texture.texture)),
    }
}

fn convert_model(model: easy_gltf::Model, reverse_texture: &mut ReverseTextureBank) -> Model {
    let triangles = model
        .triangles()
        .unwrap()
        .into_iter()
        .map(|t| t.into())
        .collect();
    let material = convert_material(model.material(), reverse_texture);
    Model::Mesh {
        triangles,
        material,
    }
}

pub fn convert_gltf_to_isf<P: AsRef<Path>>(
    input: P,
    output: P,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output = PathBuf::from(output.as_ref());
    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    } else if !output.is_dir() {
        return Err(format!("'{}' is not a directory", output.display()).into());
    }

    let scenes = easy_gltf::load(input)?;

    if scenes.is_empty() {
        return Err("No scenes found in gltf file".into());
    }

    if scenes[0].cameras.is_empty() {
        // TODO: Return error instead
        panic!("No camera found")
    }
    let camera = scenes[0].cameras[0].clone().into();

    let lights = scenes[0]
        .lights
        .clone()
        .into_iter()
        .map(|l| l.into())
        .collect();

    let mut reverse_texture = Default::default();
    let models = scenes[0]
        .models
        .clone()
        .into_iter()
        .map(|m| convert_model(m, &mut reverse_texture))
        .collect();

    let scene = Scene {
        models,
        camera,
        lights,
        ..Default::default()
    };

    // Save scene
    let file = File::create(output.join("scene.isf"))?;
    to_writer(file, &scene)?;

    // Save textures
    reverse_texture.save(&output);
    Ok(())
}

impl From<easy_gltf::Camera> for Camera {
    fn from(cam: easy_gltf::Camera) -> Self {
        let fov = match cam.projection {
            easy_gltf::Projection::Perspective { yfov, .. } => yfov.0,
            easy_gltf::Projection::Orthographic { .. } => {
                panic!("Orthographic camera not supported")
            }
        };
        Self {
            transform: cam.transform.into(),
            fov,
            zfar: cam.zfar,
            znear: cam.znear,
        }
    }
}

impl From<easy_gltf::model::Triangle> for Triangle {
    fn from(triangle: easy_gltf::model::Triangle) -> Self {
        Self(
            triangle[0].clone().into(),
            triangle[1].clone().into(),
            triangle[2].clone().into(),
        )
    }
}

impl From<easy_gltf::model::Vertex> for Vertex {
    fn from(vertex: easy_gltf::model::Vertex) -> Self {
        Self {
            position: vertex.position.into(),
            normal: vertex.normal.into(),
            tex_coords: vertex.tex_coords.into(),
        }
    }
}

impl From<easy_gltf::Light> for Light {
    fn from(light: easy_gltf::Light) -> Self {
        match light {
            easy_gltf::Light::Directional {
                direction,
                color,
                intensity,
            } => Self::Directional {
                direction: direction.into(),
                color: (color * intensity).into(),
            },
            easy_gltf::Light::Point {
                position,
                color,
                intensity,
            } => Self::Point {
                position: position.into(),
                color: (color * intensity).into(),
                size: 0.1,
            },
            easy_gltf::Light::Spot {
                position,
                color,
                intensity,
                ..
            } => Self::Point {
                position: position.into(),
                color: (color * intensity).into(),
                size: 0.1,
            },
        }
    }
}
