pub mod brdf;

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
use std::f32::consts::PI;
use std::time::Instant;
use brdf::Brdf;

pub struct Renderer {
    width: u32,
    height: u32,
    quiet: bool,
    samples: usize,
    bounces: usize,
}

impl Renderer {
    // To avoid self-intersection
    const NORMAL_BIAS: f32 = 0.00001;

    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        Renderer {
            width: config.resolution.x,
            height: config.resolution.y,
            quiet: config.quiet,
            samples: config.samples,
            bounces: config.bounces,
        }
    }

    /// Render a scene
    pub fn render<B: Brdf>(&self, scene: &Scene) -> RgbImage {
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
        let now = Instant::now();

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
                        let color = Self::render_pixel::<B>(scene, &ray, self.bounces, self.samples);
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
        if let Some(ref mut pb) = *pb.lock().unwrap() {
            pb.finish_print(format!("Done: {}s", now.elapsed().as_secs()).as_str());
        }

        // Unwrap image
        Arc::try_unwrap(image).unwrap().into_inner().unwrap()
    }

    /// Render the color of a pixel given a ray and the scene
    fn render_pixel<B: Brdf>(scene: &Scene, ray: &Ray, bounces: usize, samples: usize) -> Vector3<f32> {
        match Self::ray_cast(scene, ray) {
            None => Vector3::new(0., 0., 0.),
            Some((hit, model)) => Self::radiance::<B>(scene, model, &hit, ray, bounces, samples),
        }
    }

    fn ray_cast(scene: &Scene, ray: &Ray) -> Option<(Hit, Arc<Model>)> {
        let mut best = None;
        for model in scene.models.intersect(&ray.origin, &ray.direction) {
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

    fn radiance<B: Brdf>(
        scene: &Scene,
        model: Arc<Model>,
        hit: &Hit,
        ray_in: &Ray,
        bounces: usize,
        samples: usize,
    ) -> Vector3<f32> {
        let n = if let Some(normal) = model.material.get_normal(hit.tex_coords) {
            Self::normal_tangent_to_world(&normal, hit)
        } else {
            hit.normal
        };

        let brdf = B::new(&model.material, hit.tex_coords, n);

        let v = -1. * ray_in.direction;
        
        // Direct Light computation
        let mut direct_radiance = Vector3::zero();
        for light in scene.lights.iter() {
            let (light_radiance, light_direction) = Self::get_light_info(light, hit, scene);
            if light_radiance == Zero::zero() {
                continue;
            }

            let l = -1. * light_direction;
            direct_radiance += brdf.eval(n, v, l, light_radiance);
        }

        // Indirect light computation
        let mut indirect_radiance = Vector3::zero();
        if bounces > 0 {
            for _ in 0..samples {
                let ray_bounce_dir = brdf.sample(v);
                let ray_bounce = Ray::new(hit.position + hit.normal * Self::NORMAL_BIAS, ray_bounce_dir);

                let light_radiance = Self::render_pixel::<B>(scene, &ray_bounce, bounces - 1, samples);

                let l = ray_bounce_dir;
                let sample_radiance = brdf.eval(n, v, l, light_radiance);
                
                let pdf = brdf.pdf(n, v, l);
                let weighted_sample_radiance = sample_radiance / pdf;
                
                indirect_radiance += weighted_sample_radiance;
            }
            indirect_radiance /= samples as f32;
        }

        let mut radiance = direct_radiance + indirect_radiance;
        radiance += brdf.get_ambient_occlusion();

        // HDR
        radiance = radiance.div_element_wise(radiance + Vector3::new(1., 1., 1.));
        radiance = Vector3::new(
            radiance.x.powf(1. / 2.2),
            radiance.y.powf(1. / 2.2),
            radiance.z.powf(1. / 2.2),
        );

        radiance
    }

    fn normal_tangent_to_world(normal: &Vector3<f32>, hit: &Hit) -> Vector3<f32> {
        let edge1 = hit.triangle[1].position - hit.triangle[0].position;
        let edge2 = hit.triangle[2].position - hit.triangle[0].position;
        let delta_uv1 = hit.triangle[1].tex_coords - hit.triangle[0].tex_coords;
        let delta_uv2 = hit.triangle[2].tex_coords - hit.triangle[0].tex_coords;

        let f = 1. / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
        let tangent = Vector3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        )
        .normalize();

        let bitangent = hit.normal.cross(tangent);
        let tbn = Matrix3::from_cols(tangent, bitangent, hit.normal);
        (tbn * normal).normalize()
    }

    fn get_light_info(light: &Light, hit: &Hit, scene: &Scene) -> (Vector3<f32>, Vector3<f32>) {
        match light {
            Light::Directional {
                direction,
                color,
                intensity,
            } => {
                let shadow_ray_ori = hit.position + hit.normal * Self::NORMAL_BIAS;
                let shadow_ray_dir = -1. * direction;
                let shadow_ray = Ray::new(shadow_ray_ori, shadow_ray_dir);
                match Self::ray_cast(scene, &shadow_ray) {
                    None => (*intensity * color, direction.clone()),
                    _ => (Vector3::zero(), Vector3::zero()),
                }
            }

            Light::Point {
                position,
                color,
                intensity,
            } => {
                let direction = hit.position - position;
                let dist = direction.magnitude();
                let direction = direction.normalize();

                let shadow_ray_ori = hit.position + hit.normal * Self::NORMAL_BIAS;
                let shadow_ray_dir = -1. * direction;
                let shadow_ray = Ray::new(shadow_ray_ori, shadow_ray_dir);

                let dissipation = 4. * PI * dist * dist; // 4πr^2
                let light_dissipated = intensity / dissipation * color;

                match Self::ray_cast(scene, &shadow_ray) {
                    Some(x) => {
                        let shadow_factor = (x.0.dist / dist).min(1.); // tricked soft shadow
                        (light_dissipated * shadow_factor, direction)
                    }
                    _ => (light_dissipated, direction),
                }
            }
            _ => unimplemented!("Light not implemented: {:?}", light),
        }
    }
}
