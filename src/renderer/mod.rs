pub mod brdf;
pub mod tonemap;

mod material_sample;

pub use material_sample::MaterialSample;

use crate::Scene;
use cgmath::*;
use image::{Rgb, RgbImage};
use pbr::ProgressBar;
use std::sync::{Arc, Mutex};

use crate::config::*;
use crate::scene::model::Model;
use crate::utils::{Hit, Intersectable, Ray};
use brdf::*;
use easy_gltf::Light;
use rayon::ThreadPoolBuilder;
use std::f32::consts::PI;
use std::time::Instant;
use tonemap::*;

pub struct Renderer {
    profile: Profile,
    quiet: bool,
}

impl Renderer {
    // To avoid self-intersection
    const NORMAL_BIAS: f32 = 0.00001;

    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        Renderer {
            profile: config.profile,
            quiet: config.quiet,
        }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let width = self.profile.resolution.width;
        let height = self.profile.resolution.height;
        let mut image = RgbImage::new(width, height);

        // Save f32 cast of resolution
        let width_f = width as f32;
        let height_f = height as f32;

        let image_ratio = width_f / height_f;

        // Create progress bar (if quiet isn't activated)
        let mut pb = if self.quiet {
            None
        } else {
            let mut pb = ProgressBar::new(self.profile.samples as u64);
            pb.message("Rendering: ");
            Some(pb)
        };

        // Create thread pool
        let pool = ThreadPoolBuilder::new().num_threads(8).build().unwrap();
        let now = Instant::now();

        let buffer = vec![Vector3::<f32>::zero(); (width * height) as usize];
        let buffer = Arc::new(Mutex::new(buffer));

        for _ in 0..self.profile.samples {
            pool.scope(|s| {
                for x in 0..width {
                    for y in 0..height {
                        let buffer = buffer.clone();
                        s.spawn(move |_| {
                            let mut screen_x = x as f32 + rand::random::<f32>();
                            screen_x = screen_x / width_f * 2. - 1.;
                            screen_x *= Rad::tan(scene.camera.fov / 2.) * image_ratio;

                            let mut screen_y = y as f32 + rand::random::<f32>();
                            screen_y = 1. - screen_y / height_f * 2.;
                            screen_y *= Rad::tan(scene.camera.fov / 2.);

                            let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
                            let ray_dir = scene.camera.apply_transform_vector(&ray_dir);
                            let ray = Ray::new(scene.camera.position(), ray_dir);

                            // Compute pixel color
                            let color = self.render_pixel(scene, ray) / self.profile.samples as f32;

                            let mut buffer = buffer.lock().unwrap();
                            buffer[(x * height + y) as usize] += color;
                        });
                    }
                }
            });
            // Update progressbar
            if let Some(ref mut pb) = pb {
                pb.inc();
            }
        }

        // Final pass
        let buffer = buffer.lock().unwrap();
        for x in 0..width {
            for y in 0..height {
                // Post process
                let color = self.post_processing(buffer[(x * height + y) as usize]);

                // Set pixel color into image
                image[(x, y)] = color;
            }
        }

        if let Some(ref mut pb) = pb {
            pb.finish_print(format!("Done: {}s", now.elapsed().as_secs()).as_str());
        }

        image
    }

    /// Render the color of a pixel given a ray and the scene
    fn render_pixel(&self, scene: &Scene, mut ray: Ray) -> Vector3<f32> {
        let mut color = Vector3::new(0., 0., 0.);
        let mut throughput = Vector3::new(1., 1., 1.);

        for bounce in 0..(self.profile.bounces + 1) {
            // Test intersection
            let (hit, model) = match Self::ray_cast(scene, &ray) {
                None => {
                    let background_color: Vector3<f32> = self.profile.background_color.into();
                    return color + throughput.mul_element_wise(background_color);
                }
                Some((hit, model)) => (hit, model),
            };

            let material = MaterialSample::new(&model.material, hit.tex_coords);

            // Get normal from geometry or texture
            let n = if let Some(normal) = model.material.get_normal(hit.tex_coords) {
                Self::normal_tangent_to_world(&normal, &hit)
            } else {
                hit.normal
            };

            let view_direction = -1. * ray.direction;

            ray = self.compute_radiance(
                scene,
                &hit,
                view_direction,
                &material,
                n,
                &mut color,
                &mut throughput,
                bounce < self.profile.bounces,
            );
        }

        return color;
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

    fn compute_radiance(
        &self,
        scene: &Scene,
        hit: &Hit,
        view_direction: Vector3<f32>,
        material: &MaterialSample,
        n: Vector3<f32>,
        color: &mut Vector3<f32>,
        throughput: &mut Vector3<f32>,
        compute_indirect: bool,
    ) -> Ray {
        let mut brdf = get_brdf(&material, self.profile.brdf);

        // AO
        *color += throughput.mul_element_wise(Self::compute_ambient_occlusion(
            material.albedo,
            material.ambient_occlusion,
        ));

        // Emissive
        *color += throughput.mul_element_wise(material.emissive);

        // Direct Light computation
        for light in scene.lights.iter() {
            let (light_radiance, light_direction) = Self::get_light_info(light, &hit, scene);
            if light_radiance == Zero::zero() {
                continue;
            }
            let reversed_light_dir = -1. * light_direction;
            *color += throughput.mul_element_wise(brdf.eval_direct(
                n,
                view_direction,
                reversed_light_dir,
            )).mul_element_wise(light_radiance);
        }

        // Indirect light computation
        if compute_indirect {
            let ray_bounce = Ray::new(
                hit.position + hit.normal * Self::NORMAL_BIAS,
                brdf.sample(n, view_direction),
            );
            let sample_radiance = brdf.eval_indirect(n, view_direction, ray_bounce.direction);
            let weighted_sample_radiance = sample_radiance / brdf.pdf();
            *throughput = throughput.mul_element_wise(weighted_sample_radiance);
            ray_bounce
        } else {
            Default::default()
        }
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

    fn post_processing(&self, color: Vector3<f32>) -> Rgb<u8> {
        // HDR
        let color = tonemap(self.profile.tonemap, color);

        // Gamma correction
        let gamma = 2.2;
        let color = Vector3::new(
            color.x.powf(1. / gamma),
            color.y.powf(1. / gamma),
            color.z.powf(1. / gamma),
        );

        // Convert Vector3 into Rgb
        Rgb::from([
            (color.x * 255.) as u8,
            (color.y * 255.) as u8,
            (color.z * 255.) as u8,
        ])
    }

    fn compute_ambient_occlusion(
        albedo: Vector3<f32>,
        ambient_occlusion: Option<f32>,
    ) -> Vector3<f32> {
        if let Some(ambient_occlusion) = ambient_occlusion {
            0.03 * ambient_occlusion * albedo
        } else {
            Zero::zero()
        }
    }
}
