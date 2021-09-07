use crate::config::Resolution;
use crate::renderer::material_sample::MaterialSample;
use crate::renderer::utils::*;
use crate::utils::Ray;
use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};
use std::collections::HashMap;

pub fn debug_render(scene: &Scene, resolution: &Resolution) {
    let buffer_names = vec![
        "normal",
        "albedo",
        "metalness",
        "roughness",
        "ao",
        "emissive",
    ];

    // Create buffers
    let mut buffers = HashMap::new();
    let width = resolution.width;
    let height = resolution.height;
    for buffer_name in buffer_names {
        buffers.insert(
            buffer_name,
            vec![Vector3::<f32>::zero(); (width * height) as usize],
        );
    }

    // Fill buffers
    let width_f = width as f32;
    let height_f = height as f32;
    let image_ratio = width_f / height_f;

    for x in 0..width {
        for y in 0..height {
            let mut screen_x = x as f32 + rand::random::<f32>();
            screen_x = screen_x / width_f * 2. - 1.;
            screen_x *= Rad::tan(scene.camera.fov / 2.) * image_ratio;

            let mut screen_y = y as f32 + rand::random::<f32>();
            screen_y = 1. - screen_y / height_f * 2.;
            screen_y *= Rad::tan(scene.camera.fov / 2.);

            let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
            let ray_dir = scene.camera.apply_transform_vector(&ray_dir);
            let ray = Ray::new(scene.camera.position(), ray_dir);
            let pixels = render_debug_pixels(scene, &ray);

            for (buffer_name, pixel) in pixels {
                let index = (x * height + y) as usize;
                buffers.get_mut(buffer_name).unwrap()[index] = pixel;
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

fn render_debug_pixels<'a>(scene: &'a Scene, ray: &'a Ray) -> HashMap<&'a str, Vector3<f32>> {
    let mut result = HashMap::new();
    // Cast ray
    let (hit, model) = match ray_cast(scene, ray) {
        None => return result,
        Some(res) => res,
    };

    let material = MaterialSample::new(&model.material, hit.tex_coords);
    // Normal
    let normal = if let Some(normal) = model.material.get_normal(hit.tex_coords) {
        normal_tangent_to_world(&normal, &hit)
    } else {
        hit.normal
    };
    result.insert("normal", normal);

    // Albedo
    result.insert("albedo", material.albedo);

    let one = Vector3::new(1., 1., 1.);

    // metalness
    result.insert("metalness", one * material.metalness);

    // roughness
    result.insert("roughness", one * material.roughness);

    // ao
    result.insert("ao", one * material.ambient_occlusion.unwrap_or(0.));

    // emissive
    result.insert("emissive", material.emissive);

    result
}
