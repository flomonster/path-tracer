use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};
use pbr::ProgressBar;
use std::sync::{Arc, Mutex};

use crate::scene::model::Model;
use crate::utils::{Hit, Intersectable, Ray};
use crate::Config;
use easy_gltf::Light;
use rayon::ThreadPoolBuilder;
use std::f32::consts;

pub struct Raytracer {
    width: u32,
    height: u32,
    quiet: bool,
}

impl Raytracer {
    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        Raytracer {
            width: config.resolution.x,
            height: config.resolution.y,
            quiet: config.quiet,
        }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let image = Arc::new(Mutex::new(RgbImage::new(self.width, self.height)));

        // Save f32 cast of resolution
        let width = self.width as f32;
        let height = self.height as f32;

        let image_ratio = width / height;

        // Create progress bar (if quiet isn't activated)
        let pb = if self.quiet {
            None
        } else {
            let mut pb = ProgressBar::new((self.width * self.height) as u64);
            pb.message("Rendering: ");
            Some(pb)
        };
        let pb = Arc::new(Mutex::new(pb));

        // Create thread pool
        let pool = ThreadPoolBuilder::new().num_threads(8).build().unwrap();

        pool.scope(|s| {
            for x in 0..self.width {
                for y in 0..self.height {
                    let image = image.clone();
                    let pb = pb.clone();
                    s.spawn(move |_| {
                        let screen_x = (x as f32 + 0.5) / width * 2. - 1.;
                        let screen_x = screen_x * Rad::tan(scene.camera.fov / 2.) * image_ratio;

                        let screen_y = 1. - (y as f32 + 0.5) / height * 2.;
                        let screen_y = screen_y * Rad::tan(scene.camera.fov / 2.);

                        let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
                        let ray_dir = scene.camera.apply_transform_vector(&ray_dir);
                        let ray = Ray::new(scene.camera.position(), ray_dir);

                        // Compute pixel color
                        let color = Self::render_pixel(scene, &ray, 4);
                        // Convert Vector3 into Rgb
                        let color = Rgb::from([
                            (color.x * 255.) as u8,
                            (color.y * 255.) as u8,
                            (color.z * 255.) as u8,
                        ]);

                        // Set pixel color into image
                        image.lock().unwrap()[(x, y)] = color;

                        // Update progressbar
                        if let Some(ref mut pb) = *pb.lock().unwrap() {
                            pb.inc();
                        }
                    });
                }
            }
        });

        // Unwrap image
        Arc::try_unwrap(image).unwrap().into_inner().unwrap()
    }

    /// Render the color of a pixel given a ray and the scene
    fn render_pixel(scene: &Scene, ray: &Ray, max_bounds: i32) -> Vector3<f32> {
        if max_bounds < 0 {
            return Vector3::new(0., 0., 0.);
        }
        match Self::ray_cast(scene, ray) {
            None => Vector3::new(0., 0., 0.),
            Some((hit, model)) => Self::compute_shader(scene, model, &hit, max_bounds),
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

    fn compute_shader(scene: &Scene, model: &Model, hit: &Hit, _max_bounds: i32) -> Vector3<f32> {
        let mut global_diffuse = Vector3::new(0., 0., 0.);

        for light in scene.lights.iter() {
            let diffuse = match light {
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => Self::directional_light_diffuse(scene, hit, direction, color, *intensity),
                Light::Point {
                    position,
                    color,
                    intensity,
                } => Self::point_light_diffuse(scene, hit, position, color, *intensity),
                _ => unimplemented!(),
            };

            if let Some(diffuse) = diffuse {
                // Add diffuse to the global diffuse
                global_diffuse += diffuse;
            }
        }
        global_diffuse.x = global_diffuse.x.min(1.);
        global_diffuse.y = global_diffuse.y.min(1.);
        global_diffuse.z = global_diffuse.z.min(1.);
        model
            .material
            .get_base_color(hit.text_coords)
            .truncate()
            .mul_element_wise(global_diffuse)
    }

    fn point_light_diffuse(
        scene: &Scene,
        hit: &Hit,
        position: &Vector3<f32>,
        light_color: &Vector3<f32>,
        intensity: f32,
    ) -> Option<Vector3<f32>> {
        let mut dir = hit.position - position;
        let dist = dir.magnitude();
        dir = dir.normalize();
        let ray_shadow = Ray::new(hit.position + hit.normal * 0.0001, dir * -1.);
        if Self::ray_cast(scene, &ray_shadow).is_none() {
            let light_dissipated = 4. * consts::PI * dist * dist; // 4πr^2
            Some(light_color * intensity * hit.normal.dot(dir * -1.).max(0.) / light_dissipated)
        } else {
            None
        }
    }

    fn directional_light_diffuse(
        scene: &Scene,
        hit: &Hit,
        dir: &Vector3<f32>,
        light_color: &Vector3<f32>,
        intensity: f32,
    ) -> Option<Vector3<f32>> {
        let ray_shadow = Ray::new(hit.position + hit.normal * 0.0001, dir * -1.);
        if Self::ray_cast(scene, &ray_shadow).is_none() {
            Some(light_color * intensity * hit.normal.dot(dir * -1.).max(0.))
        } else {
            None
        }
    }
}
