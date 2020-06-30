use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};

use crate::utils::Ray;

pub struct Raytracer {
    width: u32,
    height: u32,
}

impl Raytracer {
    /// Create new raytracer given resolution
    pub fn new(width: u32, height: u32) -> Self {
        Raytracer { width, height }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let mut image = RgbImage::new(self.width, self.height);

        // Save f32 cast of resolution
        let width = self.width as f32;
        let height = self.height as f32;

        let image_ratio = width / height;

        for x in 0..self.width {
            for y in 0..self.height {
                let screen_x = (x as f32 + 0.5) / width * 2. - 1.;
                let screen_x = screen_x * Rad::tan(scene.camera.fov / 2.) * image_ratio;

                let screen_y = 1. - (y as f32 + 0.5) / height * 2.;
                let screen_y = screen_y * Rad::tan(scene.camera.fov / 2.);

                // TODO: Take camera angle into acount
                let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
                let ray = Ray::new(scene.camera.position, ray_dir);

                // Compute pixel color
                let color = self.render_pixel(scene, &ray);

                // Set pixel color into image
                image[(x, y)] = color;
            }
        }
        image
    }

    /// Render the color of a pixel given a ray and the scene
    fn render_pixel(&self, scene: &Scene, ray: &Ray) -> Rgb<u8> {
        let mut color = Vector3::new(0., 0., 0.);
        let mut best = None;
        for model in scene.models.iter() {
            if let Some((dist, triangle)) = model.intersect(ray) {
                best = match best {
                    None => Some((dist, triangle, model)),
                    Some((best_dist, _, _)) if best_dist > dist => Some((dist, triangle, model)),
                    _ => best,
                }
            }
        }

        color = match best {
            None => color,
            Some((_, _, model)) => model.material.diffuse,
        };

        // Convert Vector3 into Rgb
        Rgb::from([
            (color.x * 255.) as u8,
            (color.y * 255.) as u8,
            (color.z * 255.) as u8,
        ])
    }
}
