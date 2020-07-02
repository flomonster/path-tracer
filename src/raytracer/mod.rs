use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};
use std::sync::{Arc, Mutex};

use crate::scene::model::Model;
use crate::scene::Light;
use crate::utils::{Hit, Intersectable, Ray};
use crate::Config;
use rayon::ThreadPoolBuilder;

pub struct Raytracer {
    width: u32,
    height: u32,
}

impl Raytracer {
    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        Raytracer {
            width: config.resolution.x,
            height: config.resolution.y,
        }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let image = Arc::new(Mutex::new(RgbImage::new(self.width, self.height)));

        // Save f32 cast of resolution
        let width = self.width as f32;
        let height = self.height as f32;

        let image_ratio = width / height;

        // Create thread pool
        let pool = ThreadPoolBuilder::new().num_threads(8).build().unwrap();

        pool.scope(|s| {
            for x in 0..self.width {
                for y in 0..self.height {
                    let image = image.clone();
                    s.spawn(move |_| {
                        let screen_x = (x as f32 + 0.5) / width * 2. - 1.;
                        let screen_x = screen_x * Rad::tan(scene.camera.fov / 2.) * image_ratio;

                        let screen_y = 1. - (y as f32 + 0.5) / height * 2.;
                        let screen_y = screen_y * Rad::tan(scene.camera.fov / 2.);

                        // TODO: Take camera angle into acount
                        let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
                        let ray = Ray::new(scene.camera.position, ray_dir);

                        // Compute pixel color
                        let color = Self::render_pixel(scene, &ray);

                        // Set pixel color into image
                        let mut image = image.lock().unwrap();
                        image[(x, y)] = color;
                    });
                }
            }
        });

        // Unwrap image
        Arc::try_unwrap(image).unwrap().into_inner().unwrap()
    }

    /// Render the color of a pixel given a ray and the scene
    fn render_pixel(scene: &Scene, ray: &Ray) -> Rgb<u8> {
        let color = match Self::ray_cast(scene, ray) {
            None => Vector3::new(0., 0., 0.),
            Some((hit, model)) => Self::compute_shader(scene, model, &hit),
        };

        // Convert Vector3 into Rgb
        Rgb::from([
            (color.x * 255.) as u8,
            (color.y * 255.) as u8,
            (color.z * 255.) as u8,
        ])
    }

    fn ray_cast<'a>(scene: &'a Scene, ray: &Ray) -> Option<(Hit, &'a Model)> {
        let mut best = None;
        for model in scene.models.iter() {
            if let Some(hit) = model.intersect(ray) {
                best = match best {
                    None => Some((hit, model)),
                    Some((best, _)) if best.dist > hit.dist => Some((hit, model)),
                    _ => best,
                }
            }
        }
        best
    }

    fn compute_shader(scene: &Scene, model: &Model, hit: &Hit) -> Vector3<f32> {
        let mut color = Vector3::new(0., 0., 0.);
        let hit_normal = (1. - hit.uv.x - hit.uv.y) * hit.triangle.0.normal
            + hit.uv.x * hit.triangle.1.normal
            + hit.uv.y * hit.triangle.2.normal;

        for light in scene.lights.iter() {
            match light {
                Light::Directional(dir, light_color, intensity) => {
                    let ray = Ray::new(hit.position + hit_normal * 0.0001, dir * -1.);
                    if Self::ray_cast(scene, &ray).is_none() {
                        color += model.material.diffuse.mul_element_wise(
                            light_color * (*intensity) * hit_normal.dot(dir * -1.).max(0.),
                        );
                    }
                }
                _ => unimplemented!("Point light compute shader"),
            }
        }
        color
    }
}
