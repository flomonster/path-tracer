pub mod brdf;
pub mod debug_renderer;
pub mod tonemap;
pub mod utils;

mod material_sample;
mod viewer;

use crate::config::*;
use crate::utils::{Hit, Ray};
use crate::Scene;
use brdf::*;
use cgmath::*;
use easy_gltf::Light;
use image::{Rgb, RgbImage};
use material_sample::MaterialSample;
use pbr::ProgressBar;
use rayon::ThreadPoolBuilder;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tonemap::*;
use utils::*;
use viewer::Viewer;

pub struct Renderer {
    profile: Profile,
    quiet: bool,
    viewer: Option<Viewer>,
}

impl Renderer {
    // To avoid self-intersection
    const NORMAL_BIAS: f32 = 0.00001;

    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        let viewer = if config.viewer {
            Some(Viewer::create(config.profile.resolution.clone()))
        } else {
            None
        };

        Renderer {
            profile: config.profile,
            quiet: config.quiet,
            viewer,
        }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let width = self.profile.resolution.width;
        let height = self.profile.resolution.height;

        // Buffer containing the rendered image
        let buffer = vec![Vector3::<f32>::zero(); (width * height) as usize];
        let buffer = Arc::new(Mutex::new(buffer));

        // Create progress bar (if quiet isn't activated)
        let mut pb = if self.quiet {
            None
        } else {
            let mut pb = ProgressBar::new(self.profile.samples as u64);
            pb.message("Rendering: ");
            pb.set(0);
            Some(pb)
        };

        // Save f32 cast of resolution
        let width_f = width as f32;
        let height_f = height as f32;

        let image_ratio = width_f / height_f;

        let profile = self.profile;
        let sender = Arc::new(Mutex::new(if let Some(viewer) = &self.viewer {
            Some(viewer.sender.clone())
        } else {
            None
        }));

        // Create thread pool
        let pool = ThreadPoolBuilder::new()
            .num_threads(profile.nb_treads)
            .build()
            .unwrap();

        let now = Instant::now();

        for current_sample in 1..(profile.samples + 1) {
            pool.scope(|s| {
                for x in 0..width {
                    for y in 0..height {
                        let buffer = buffer.clone();
                        let sender = sender.clone();
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
                            let color = Self::render_pixel(&profile, scene, ray);

                            // Update buffer
                            let mut buffer = buffer.lock().unwrap();
                            let buffer_pos = (x * height + y) as usize;
                            buffer[buffer_pos] += color;

                            // Send it to viewer
                            let mut sender_guard = sender.lock().unwrap();
                            if let Some(sender) = &*sender_guard {
                                let color = buffer[buffer_pos] / current_sample as f32;
                                let color = Self::post_processing(&profile, color);
                                if let Err(_) = Viewer::send_pixel_update(sender, x, y, color.0) {
                                    *sender_guard = None;
                                }
                            }
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
        let mut image = RgbImage::new(width, height);
        let buffer = buffer.lock().unwrap();
        for x in 0..width {
            for y in 0..height {
                // Post process
                let color = Self::post_processing(
                    &self.profile,
                    buffer[(x * height + y) as usize] / profile.samples as f32,
                );

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
    fn render_pixel(profile: &Profile, scene: &Scene, mut ray: Ray) -> Vector3<f32> {
        let mut color = Vector3::new(0., 0., 0.);
        let mut throughput = Vector3::new(1., 1., 1.);

        for bounce in 0..(profile.bounces + 1) {
            // Test intersection
            let (hit, model) = match ray_cast(scene, &ray) {
                None => {
                    let background_color: Vector3<f32> = profile.background_color.into();
                    return color + throughput.mul_element_wise(background_color);
                }
                Some(res) => res,
            };

            let material = MaterialSample::new(&model.material, hit.tex_coords);

            // Get normal from geometry or texture
            let n = if let Some(normal) = model.material.get_normal(hit.tex_coords) {
                normal_tangent_to_world(&normal, &hit)
            } else {
                hit.normal
            };

            let view_direction = -1. * ray.direction;

            ray = Self::compute_radiance(
                profile,
                scene,
                &hit,
                view_direction,
                &material,
                n,
                &mut color,
                &mut throughput,
                bounce < profile.bounces,
            );
        }

        return color;
    }

    fn compute_radiance(
        profile: &Profile,
        scene: &Scene,
        hit: &Hit,
        view_direction: Vector3<f32>,
        material: &MaterialSample,
        n: Vector3<f32>,
        color: &mut Vector3<f32>,
        throughput: &mut Vector3<f32>,
        compute_indirect: bool,
    ) -> Ray {
        let mut brdf = get_brdf(&material, profile.brdf);

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
            *color += throughput
                .mul_element_wise(brdf.eval_direct(n, view_direction, reversed_light_dir))
                .mul_element_wise(light_radiance);
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
                match ray_cast(scene, &shadow_ray) {
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

                match ray_cast(scene, &shadow_ray) {
                    None => (light_dissipated, direction),
                    _ => (Vector3::zero(), Vector3::zero()),
                }
            }
            _ => unimplemented!("Light not implemented: {:?}", light),
        }
    }

    fn post_processing(profile: &Profile, color: Vector3<f32>) -> Rgb<u8> {
        // HDR
        let color = tonemap(profile.tonemap, color);

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

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Some(viewer) = self.viewer.take() {
            viewer.wait_for_close();
        }
    }
}
