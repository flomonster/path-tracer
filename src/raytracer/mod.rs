use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};
use std::sync::{Arc, Mutex};

use crate::scene::model::Model;
use crate::scene::Light;
use crate::utils;
use crate::utils::{Hit, Intersectable, Ray};
use crate::Config;
use rayon::ThreadPoolBuilder;
use std::f32::consts;

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
                        let ray_dir = Vector3::new(screen_x, screen_y, -10.).normalize();
                        let ray = Ray::new(scene.camera.position, ray_dir);

                        // Compute pixel color
                        let color = Self::render_pixel(scene, &ray);
                        // Convert Vector3 into Rgb
                        let color = Rgb::from([
                            (color.x * 255.) as u8,
                            (color.y * 255.) as u8,
                            (color.z * 255.) as u8,
                        ]);

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
    fn render_pixel(scene: &Scene, ray: &Ray) -> Vector3<f32> {
        match Self::ray_cast(scene, ray) {
            None => Vector3::new(0., 0., 0.),
            Some((hit, model)) => Self::compute_shader(scene, model, ray, &hit),
        }
    }

    fn ray_cast<'a>(scene: &'a Scene, ray: &Ray) -> Option<(Hit, &'a Model)> {
        let mut best = None;
        for model in scene.models.iter() {
            if let Some(hit) = model.intersect(ray) {
                best = match best {
                    None => Some((hit, model)),
                    Some((best_hit, _)) if best_hit.dist > hit.dist => Some((hit, model)),
                    _ => best,
                }
            }
        }
        best
    }

    fn compute_shader(scene: &Scene, model: &Model, ray: &Ray, hit: &Hit) -> Vector3<f32> {
        let mut color = Vector3::new(0., 0., 0.);
        let hit_normal = hit.normal();

        let illum = model.material.illumination.unwrap_or(2);

        for light in scene.lights.iter() {
            let shaders = match light {
                Light::Directional(dir, light_color, intensity) => {
                    Self::compute_shaders_directional_light(
                        scene,
                        &hit_normal,
                        hit,
                        ray,
                        model,
                        dir,
                        light_color,
                        *intensity,
                    )
                }
                Light::Point(position, light_color, intensity) => {
                    Self::compute_shaders_point_light(
                        scene,
                        &hit_normal,
                        hit,
                        ray,
                        model,
                        position,
                        light_color,
                        *intensity,
                    )
                }
            };

            if let Some((diffuse, specular)) = shaders {
                // Add diffuse
                color += model.material.get_diffuse(hit).mul_element_wise(diffuse);
                // Add specular
                color += specular * model.material.get_specular(hit);
            }
        }

        // Reflection
        if illum == 2 {
            let dir_reflected = utils::reflection(&ray.direction, &hit_normal);
            let ray_reflected = Ray::new(hit.position + hit_normal * 0.0001, dir_reflected);
            color += 0.8 * Self::render_pixel(scene, &ray_reflected);
        }

        color
    }

    fn compute_shaders_point_light(
        scene: &Scene,
        hit_normal: &Vector3<f32>,
        hit: &Hit,
        ray: &Ray,
        model: &Model,
        position: &Vector3<f32>,
        light_color: &Vector3<f32>,
        intensity: f32,
    ) -> Option<(Vector3<f32>, f32)> {
        let mut dir = hit.position - position;
        let dist = dir.magnitude();
        dir = dir.normalize();
        let ray_shadow = Ray::new(hit.position + hit_normal * 0.0001, dir * -1.);
        if Self::ray_cast(scene, &ray_shadow).is_none() {
            let light_dissipated = 4. * consts::PI * dist * dist; // 4πr^2
            Some((
                // Diffuse
                light_color * intensity * hit_normal.dot(dir * -1.).max(0.) / light_dissipated,
                // Specular
                intensity / light_dissipated
                    * (ray.direction * -1.)
                        .dot(utils::reflection(&dir, &hit_normal))
                        .max(0.)
                        .powf(model.material.get_shininess(hit)),
            ))
        } else {
            None
        }
    }

    fn compute_shaders_directional_light(
        scene: &Scene,
        hit_normal: &Vector3<f32>,
        hit: &Hit,
        ray: &Ray,
        model: &Model,
        dir: &Vector3<f32>,
        light_color: &Vector3<f32>,
        intensity: f32,
    ) -> Option<(Vector3<f32>, f32)> {
        let ray_shadow = Ray::new(hit.position + hit_normal * 0.0001, dir * -1.);
        if Self::ray_cast(scene, &ray_shadow).is_none() {
            Some((
                // Diffuse
                light_color * intensity * hit_normal.dot(dir * -1.).max(0.),
                // Specular
                intensity
                    * (ray.direction * -1.)
                        .dot(utils::reflection(&dir, &hit_normal))
                        .max(0.)
                        .powf(model.material.get_shininess(hit)),
            ))
        } else {
            None
        }
    }
}
