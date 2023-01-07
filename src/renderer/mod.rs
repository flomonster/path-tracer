pub mod brdf;
pub mod debug_renderer;
mod hit;
mod material_sample;
mod ray;
pub mod tonemap;
pub mod utils;
mod viewer;

use crate::config::*;
use crate::scene::internal::Light;
use crate::Scene;
use brdf::*;
use cgmath::*;
use derivative::Derivative;
use image::{Rgb, RgbImage};
use material_sample::MaterialSample;
use pbr::ProgressBar;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::f32::consts::PI;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tonemap::*;
use utils::*;
use viewer::Viewer;

pub use hit::Hit;
pub use ray::{Intersectable, Ray};

pub struct Renderer {
    profile: Profile,
    quiet: bool,
    viewer: Option<Viewer>,
}

#[derive(Derivative)]
#[derivative(Default)]
struct RadianceInfo {
    #[derivative(Default(value = "Vector3::zero()"))]
    color: Vector3<f32>,
    #[derivative(Default(value = "Vector3::new(1., 1., 1.)"))]
    throughput: Vector3<f32>,
}

struct SurfaceInfo {
    hit: Hit,
    material: MaterialSample,
    normal: Vector3<f32>,
}

impl Renderer {
    // To avoid self-intersection
    const NORMAL_BIAS: f32 = 0.00001;

    /// Create new raytracer given resolution
    pub fn new(config: &RenderConfig, profile: Profile) -> Self {
        let viewer = if config.viewer {
            Some(Viewer::create(profile.resolution))
        } else {
            None
        };

        Renderer {
            profile,
            quiet: config.quiet,
            viewer,
        }
    }

    /// Render a scene
    pub fn render(&self, scene: &Scene) -> RgbImage {
        let width = self.profile.resolution.width;
        let height = self.profile.resolution.height;

        // Buffer containing the rendered image
        let mut buffer = vec![Vector3::<f32>::zero(); (width * height) as usize];

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
        let viewer_enabled = AtomicBool::new(self.viewer.is_some());
        let sender = Arc::new(Mutex::new(self.viewer.as_ref().map(|v| v.sender.clone())));

        let now = Instant::now();

        for current_sample in 1..(profile.samples + 1) {
            buffer.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                let x = i as u32 % width;
                let y = i as u32 / width;

                let mut rand_gen = StdRng::seed_from_u64(current_sample as u64 + i as u64 * profile.samples as u64);

                let mut screen_x = x as f32 + rand_gen.gen::<f32>();
                screen_x = screen_x / width_f * 2. - 1.;
                screen_x *= Rad::tan(scene.camera.fov / 2.) * image_ratio;

                let mut screen_y = y as f32 + rand_gen.gen::<f32>();
                screen_y = 1. - screen_y / height_f * 2.;
                screen_y *= Rad::tan(scene.camera.fov / 2.);

                let ray_dir = Vector3::new(screen_x, screen_y, -1.).normalize();
                let ray_dir = scene.camera.apply_transform_vector(&ray_dir);
                let ray = Ray::new(scene.camera.position(), ray_dir);

                // Compute pixel color
                let color = Self::render_pixel(&profile, scene, ray, &mut rand_gen);

                // Update my pixel
                *pixel += color;

                // Send it to viewer
                if viewer_enabled.load(Ordering::Relaxed) {
                    let sender_guard = sender.lock().unwrap();
                    let sender = sender_guard.as_ref().unwrap();
                    let color = *pixel / current_sample as f32;
                    let color = Self::post_processing(&profile, color);
                    if Viewer::send_pixel_update(sender, x, y, color.0).is_err() {
                        viewer_enabled.store(false, Ordering::Relaxed);
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
        for x in 0..width {
            for y in 0..height {
                // Post process
                let color = Self::post_processing(
                    &self.profile,
                    buffer[(x + y * width) as usize] / profile.samples as f32,
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
    fn render_pixel(profile: &Profile, scene: &Scene, mut ray: Ray, rand_gen : &mut StdRng) -> Vector3<f32> {
        let mut rad_info = RadianceInfo::default();

        for bounce in 0..(profile.bounces + 1) {
            // Test intersection
            let intersections = ray_cast(scene, &ray);
            // Check if we hit nothing (background)
            if intersections.is_empty() {
                return rad_info.color + rad_info.throughput.mul_element_wise(scene.background);
            }

            let mut surface_info = None;
            for (hit, model) in intersections {
                let material_sample = hit.get_material_sample(&model);
                let normal = hit.get_normal(model.get_material());
                let opacity = material_sample.opacity;

                surface_info = Some(SurfaceInfo {
                    hit,
                    material: material_sample,
                    normal,
                });

                // Alpha transparency
                if opacity >= 1. || rand_gen.gen::<f32>() < opacity {
                    // Consider the surface as opaque and stop iterating over the intersections
                    break;
                }
            }

            let view_direction = -1. * ray.direction;

            (rad_info, ray) = Self::compute_radiance(
                profile,
                scene,
                rad_info,
                &surface_info.unwrap(),
                view_direction,
                bounce < profile.bounces,
                rand_gen,
            );

            if rad_info.throughput.magnitude2() < 0.00001 {
                return rad_info.color;
            }

            if bounce > 3 && russian_roulette(&mut rad_info.throughput, rand_gen) {
                return rad_info.color;
            }
        }
        rad_info.color
    }

    fn compute_radiance(
        profile: &Profile,
        scene: &Scene,
        rad_info: RadianceInfo,
        surface_info: &SurfaceInfo,
        view_direction: Vector3<f32>,
        compute_indirect: bool,
        rand_gen : &mut StdRng,
    ) -> (RadianceInfo, Ray) {
        let mut brdf = get_brdf(&surface_info.material, profile.brdf);
        let mut color = rad_info.color;
        let mut throughput = rad_info.throughput;
        let mut ray = Default::default();

        // Emissive
        color += throughput.mul_element_wise(surface_info.material.emissive);

        // Direct Light computation
        for light in scene.lights.iter() {
            let (light_radiance, light_direction) =
                Self::get_light_info(light, &surface_info.hit, scene);
            if light_radiance == Zero::zero() {
                continue;
            }
            let reversed_light_dir = -1. * light_direction;
            color += throughput
                .mul_element_wise(brdf.eval_direct(
                    surface_info.normal,
                    view_direction,
                    reversed_light_dir,
                ))
                .mul_element_wise(light_radiance);
        }

        // Indirect light computation
        if compute_indirect {
            ray = Ray::new(
                surface_info.hit.get_position()
                    + surface_info.hit.get_geometric_normal() * Self::NORMAL_BIAS,
                brdf.sample(surface_info.normal, view_direction, rand_gen),
            );
            let sample_radiance =
                brdf.eval_indirect(surface_info.normal, view_direction, ray.direction);
            let weighted_sample_radiance = sample_radiance / brdf.pdf();
            throughput = throughput.mul_element_wise(weighted_sample_radiance);
        }

        (RadianceInfo { color, throughput }, ray)
    }

    /// Get the light radiance and direction
    fn get_light_info(light: &Light, hit: &Hit, scene: &Scene) -> (Vector3<f32>, Vector3<f32>) {
        match light {
            Light::Directional { direction, color } => {
                let shadow_ray_ori =
                    hit.get_position() + hit.get_geometric_normal() * Self::NORMAL_BIAS;
                let shadow_ray_dir = -1. * direction;
                let shadow_ray = Ray::new(shadow_ray_ori, shadow_ray_dir);

                // TODO: no shadow for inner transparent objects
                // Atenuate the light by the opacity of occluders objects
                let mut color = *color;
                for (shadow_hit, shadow_model) in ray_cast(scene, &shadow_ray) {
                    let material_sample = shadow_hit.get_material_sample(&shadow_model);
                    color *= 1. - material_sample.opacity;
                    if color.sum() == 0. {
                        break;
                    }
                }
                (color, *direction)
            }

            Light::Point {
                position,
                color,
                size: _,
            } => {
                let direction = hit.get_position() - position;
                let dist = direction.magnitude();
                let direction = direction.normalize();

                let shadow_ray_ori =
                    hit.get_position() + hit.get_geometric_normal() * Self::NORMAL_BIAS;
                let shadow_ray_dir = -1. * direction;
                let shadow_ray = Ray::new(shadow_ray_ori, shadow_ray_dir);

                let dissipation = 4. * PI * dist * dist; // 4πr^2

                // Atenuate the light by the opacity of occluders objects
                let mut light_dissipated = color / dissipation;
                for (shadow_hit, shadow_model) in ray_cast(scene, &shadow_ray) {
                    if (shadow_hit.get_position() - hit.get_position()).magnitude() > dist {
                        // The intersected object is behind the light
                        break;
                    }
                    let material_sample = hit.get_material_sample(&shadow_model);
                    light_dissipated *= 1. - material_sample.opacity;
                    if light_dissipated.sum() == 0. {
                        break;
                    }
                }
                (light_dissipated, direction)
            }
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
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Some(viewer) = self.viewer.take() {
            viewer.wait_for_close();
        }
    }
}
