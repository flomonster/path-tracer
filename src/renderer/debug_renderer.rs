use crate::renderer::material_sample::MaterialSample;
use crate::renderer::utils::*;
use crate::{config::Resolution, scene::internal::Scene};
use cgmath::*;
use image::{Rgb, RgbImage};
use std::collections::HashMap;

use super::Hit;
use super::Ray;

pub fn debug_render(scene: &Scene, resolution: Resolution) {
    // Create buffers
    let mut buffers = HashMap::new();
    let width = resolution.width;
    let height = resolution.height;

    // Fill buffers
    let width_f = width as f32;
    let height_f = height as f32;
    let image_ratio = width_f / height_f;

    for x in 0..width {
        for y in 0..height {
            let mut screen_x = x as f32 + 0.5;
            screen_x = screen_x / width_f * 2. - 1.;
            screen_x *= Rad::tan(scene.camera.fov / 2.) * image_ratio;

            let mut screen_y = y as f32 + 0.5;
            screen_y = 1. - screen_y / height_f * 2.;
            screen_y *= Rad::tan(scene.camera.fov / 2.);

            let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
            let ray_dir = scene.camera.apply_transform_vector(&ray_dir);
            let ray = Ray::new(scene.camera.position(), ray_dir);
            let pixels = render_debug_pixels(scene, &ray);

            for (buffer_name, pixel) in pixels {
                let index = (x * height + y) as usize;
                buffers
                    .entry(buffer_name)
                    .or_insert_with(|| vec![Vector3::<f32>::zero(); (width * height) as usize])
                    [index] = pixel;
            }
        }
    }

    // Save images
    for (buffer_name, buffer) in buffers {
        let mut image = RgbImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let pixel = buffer[(x * height + y) as usize];
                image[(x, y)] = Rgb::from([
                    (pixel.x * 255.) as u8,
                    (pixel.y * 255.) as u8,
                    (pixel.z * 255.) as u8,
                ]);
            }
        }
        image.save(format!("{}.png", buffer_name)).unwrap();
    }
}

fn render_debug_pixels(scene: &Scene, ray: &Ray) -> HashMap<&'static str, Vector3<f32>> {
    let mut result = HashMap::new();
    // Cast ray
    let (hit, model) = match ray_cast(scene, ray) {
        res if res.is_empty() => return result,
        res => res[0].clone(),
    };

    let material = match hit {
        Hit::Sphere { .. } => MaterialSample::simple(model.get_material()),
        Hit::Triangle { tex_coords, .. } => MaterialSample::new(model.get_material(), &tex_coords),
    };

    // Normal
    let normal = hit.get_normal(model.get_material());
    result.insert("normal", (normal * 0.5).add_element_wise(0.5));

    // Albedo
    result.insert("albedo", material.albedo);

    let one = Vector3::new(1., 1., 1.);

    // Opacity
    result.insert("opacity", one * material.opacity);

    // metalness
    result.insert("metalness", one * material.metalness);

    // roughness
    result.insert("roughness", one * material.roughness);

    // opacity
    result.insert("opacity", one * material.opacity);

    // emissive
    result.insert("emissive", material.emissive);

    // ior
    result.insert("ior", one * material.ior / 3.);

    result
}
