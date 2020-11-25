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

pub struct Renderer {
    width: u32,
    height: u32,
    quiet: bool,
}

impl Renderer {
    /// Create new raytracer given resolution
    pub fn new(config: &Config) -> Self {
        Renderer {
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
        if let Some(ref mut pb) = *pb.lock().unwrap() {
            pb.finish_print("Done!");
        }

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
            Some((hit, model)) => Self::radiance(scene, model, &hit, ray, max_bounds),
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

    fn radiance(
        scene: &Scene,
        model: Arc<Model>,
        hit: &Hit,
        ray_in: &Ray,
        _max_bounds: i32,
    ) -> Vector3<f32> {
        let mut radiance = Vector3::zero();

        let metalness = model.material.get_metallic(hit.tex_coords);
        let roughness = model.material.get_roughness(hit.tex_coords);
        let albedo = model.material.get_base_color(hit.tex_coords);

        let n = if let Some(normal) = model.material.get_normal(hit.tex_coords) {
            Self::normal_tangent_to_world(&normal, hit)
        } else {
            hit.normal
        };

        let v = -1. * ray_in.direction;

        let f0 = Vector3::new(0.04, 0.04, 0.04);
        let f0 = f0 * (1. - metalness) + albedo * metalness;

        for light in scene.lights.iter() {
            let (light_radiance, light_direction) = Self::get_light_info(light, hit);
            let l = -1. * light_direction;
            let halfway = (v + l).normalize();

            let d = Self::distribution_ggx(n, halfway, roughness);
            let g = Self::geometry_smith(n, v, l, roughness);
            let f = Self::fresnel_schlick(halfway.dot(v).max(0.), f0);

            // Specular
            let specular = (d * f * g) / (4. * n.dot(v).max(0.) * n.dot(l).max(0.)).max(0.001);

            // Diffuse
            let kd = Vector3::new(1. - f.x, 1. - f.y, 1. - f.z) * (1. - metalness);
            let diffuse = kd.mul_element_wise(albedo) / PI;

            radiance += (diffuse + specular).mul_element_wise(light_radiance) * n.dot(l).max(0.);
        }

        if let Some(ao) = model.material.get_occlusion(hit.tex_coords) {
            radiance += 0.03 * ao * albedo;
        }

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

    fn fresnel_schlick(cos_theta: f32, f0: Vector3<f32>) -> Vector3<f32> {
        f0 + (Vector3::new(1. - f0.x, 1. - f0.y, 1. - f0.z)) * (1. - cos_theta).powi(5)
    }

    fn geometry_schlick_ggx(n_dot_v: f32, k: f32) -> f32 {
        let num = n_dot_v;
        let denom = n_dot_v * (1. - k) + k;

        num / denom
    }

    fn geometry_smith(n: Vector3<f32>, v: Vector3<f32>, l: Vector3<f32>, a: f32) -> f32 {
        let k = (a + 1.).powi(2) / 8.;
        let n_dot_v = n.dot(v).max(0.);
        let n_dot_l = n.dot(l).max(0.);
        let ggx1 = Self::geometry_schlick_ggx(n_dot_v, k);
        let ggx2 = Self::geometry_schlick_ggx(n_dot_l, k);

        return ggx1 * ggx2;
    }

    fn distribution_ggx(n: Vector3<f32>, h: Vector3<f32>, a: f32) -> f32 {
        let a2 = a.powi(4);
        let n_dot_h = n.dot(h).max(0.); //max(dot(N, H), 0.0);
        let n_dot_h_2 = n_dot_h * n_dot_h;

        let num = a2;
        let mut denom = n_dot_h_2 * (a2 - 1.) + 1.;
        denom = PI * denom * denom;

        return num / denom;
    }

    fn get_light_info(light: &Light, hit: &Hit) -> (Vector3<f32>, Vector3<f32>) {
        match light {
            Light::Directional {
                direction,
                color,
                intensity,
            } => (*intensity * color, direction.clone()),

            Light::Point {
                position,
                color,
                intensity,
            } => {
                let direction = hit.position - position;
                let dist = direction.magnitude();
                let direction = direction.normalize();
                let light_dissipated = 4. * PI * dist * dist; // 4πr^2
                (intensity / light_dissipated * color, direction)
            }
            _ => unimplemented!("Light not implemented: {:?}", light),
        }
    }
}
