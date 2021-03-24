use super::transform_to_world;
use crate::renderer::brdf::Brdf;
use crate::renderer::MaterialSample;
use crate::utils::reflection;
use cgmath::*;
use std::f32::consts::PI;

pub struct CookTorrance {
    metalness: f32,
    roughness: f32,
    albedo: Vector3<f32>,
    ambient_occlusion: Vector3<f32>,
    emissive: Vector3<f32>,
    f0: Vector3<f32>, // surface reflection at zero incidence from Fresnel-Schlick approximation
    microfacet_normal: Vector3<f32>, // a.k.a. Wm
}

impl Default for CookTorrance {
    fn default() -> Self {
        CookTorrance {
            metalness: 0.,
            roughness: 0.,
            albedo: Zero::zero(),
            emissive: Zero::zero(),
            ambient_occlusion: Zero::zero(),
            f0: Zero::zero(),
            microfacet_normal: Zero::zero(),
        }
    }
}

impl Brdf for CookTorrance {
    fn new(material: &MaterialSample) -> Self {
        Self {
            metalness: material.metalness,
            roughness: material.roughness,
            albedo: material.albedo,
            emissive: material.emissive,
            ambient_occlusion: Self::compute_ambient_occlusion(
                material.albedo,
                material.ambient_occlusion,
            ),
            f0: Self::compute_f0(material.metalness, material.albedo),
            microfacet_normal: Zero::zero(),
        }
    }

    fn sample(&mut self, geometric_normal: Vector3<f32>, v: Vector3<f32>) -> Vector3<f32> {
        // Compute a new random microfacet normal
        self.compute_microfacet_normal(geometric_normal);

        // Compute direction by reflecting v about the microfacet normal
        let sample_dir = reflection(&v, &self.microfacet_normal);
        return sample_dir.normalize();
    }

    fn eval_direct(
        &self,
        geometric_normal: Vector3<f32>, // triangle normal or texture normal
        view_direction: Vector3<f32>,   // from hit point to the viewer
        light_direction: Vector3<f32>,  // from hit point to the light
        light_radiance: Vector3<f32>,
    ) -> Vector3<f32> {
        let halfway = (view_direction + light_direction).normalize();
        let d = self.distribution_ggx(geometric_normal, halfway);
        let f = self.fresnel_schlick(halfway.dot(view_direction).max(0.));
        let g = self.geometry_smith(geometric_normal, view_direction, light_direction);

        // Specular
        let specular = (d * f * g)
            / (4.
                * geometric_normal.dot(view_direction).max(0.)
                * geometric_normal.dot(light_direction).max(0.))
            .max(0.0001);
        let cosine_term = geometric_normal.dot(light_direction).max(0.);
        let specular = specular.mul_element_wise(light_radiance) * cosine_term;

        // Diffuse
        let diffuse = self.compute_diffuse(f, geometric_normal, light_direction, light_radiance);

        return diffuse + specular + self.emissive;
    }

    fn eval_indirect(
        &self,
        geometric_normal: Vector3<f32>, // triangle normal or texture normal
        view_direction: Vector3<f32>,   // from hit point to the viewer
        light_direction: Vector3<f32>,  // from hit point to the light
        light_radiance: Vector3<f32>,
    ) -> Vector3<f32> {
        let halfway = (view_direction + light_direction).normalize();
        let f = self.fresnel_schlick(halfway.dot(view_direction).max(0.));
        let g = self.geometry_smith(geometric_normal, view_direction, light_direction);

        // Specular
        let specular = if geometric_normal.dot(light_direction) > 0.
            && light_direction.dot(self.microfacet_normal) > 0.
        {
            let weight_num = view_direction.dot(self.microfacet_normal).abs();
            let weight_denom = view_direction.dot(geometric_normal).abs()
                * self.microfacet_normal.dot(geometric_normal).abs();
            let weight = weight_num / weight_denom;
            (f * g * weight).mul_element_wise(light_radiance) // NDF and cosine factor are canceled by PDF
        } else {
            // If our sample is not in the upper hemisphere
            Zero::zero()
        };

        // Diffuse
        let diffuse = self.compute_diffuse(f, geometric_normal, light_direction, light_radiance);

        return diffuse + specular + self.emissive;
    }

    fn pdf(&self) -> f32 {
        // We simplify the PDF by canceling the NDF term in the BRDF
        return 1.;
    }

    fn get_ambient_occlusion(&self) -> Vector3<f32> {
        self.ambient_occlusion
    }
}

impl CookTorrance {
    // Lambertian diffuse
    fn compute_diffuse(
        &self,
        ks: Vector3<f32>, // specular ratio, equivalent to fresnel ratio in Cook Torrance
        geometric_normal: Vector3<f32>,
        light_direction: Vector3<f32>,
        light_radiance: Vector3<f32>,
    ) -> Vector3<f32> {
        let kd =
            Vector3::new(1. - ks.x, 1. - ks.y, 1. - ks.z) * (1. - self.metalness);
        let diffuse = kd.mul_element_wise(self.albedo) / PI;
        diffuse.mul_element_wise(light_radiance) * geometric_normal.dot(light_direction).max(0.)
    }

    fn compute_microfacet_normal(&mut self, geometric_normal: Vector3<f32>) {
        let a = self.roughness * self.roughness;
        let a2 = a * a;
        // Generate uniform random variables between 0 and 1
        let r1: f32 = rand::random();
        let r2: f32 = rand::random();

        // Compute spherical coordinates of the normal
        // Theta depends on the roughness according to the NDF (due to importance sampling on microfacet model)
        let theta = (((1. - r1) / (r1 * (a2 - 1.) + 1.)).sqrt()).acos();
        // Phi can be sampled uniformly because the NDF is isotropic
        let phi = 2. * PI * r2;

        // Convert to cartesian coordinates
        let sin_theta = theta.sin();
        let x = sin_theta * phi.cos();
        let y = theta.cos();
        let z = sin_theta * phi.sin();
        let microfacet_normal = Vector3::new(x, y, z).normalize();

        self.microfacet_normal = transform_to_world(microfacet_normal, geometric_normal).normalize();
    }

    fn fresnel_schlick(&self, cos_theta: f32) -> Vector3<f32> {
        self.f0
            + (Vector3::new(1. - self.f0.x, 1. - self.f0.y, 1. - self.f0.z))
                * (1. - cos_theta).powi(5)
    }

    fn geometry_schlick_ggx(&self, n_dot_v: f32, k: f32) -> f32 {
        let num = n_dot_v;
        let denom = n_dot_v * (1. - k) + k;

        num / denom
    }

    fn geometry_smith(&self, n: Vector3<f32>, v: Vector3<f32>, l: Vector3<f32>) -> f32 {
        let a = self.roughness;
        let n_dot_v = n.dot(v).max(0.);
        let n_dot_l = n.dot(l).max(0.);

        let k = (a + 1.).powi(2) / 8.;
        let ggx1 = self.geometry_schlick_ggx(n_dot_v, k);
        let ggx2 = self.geometry_schlick_ggx(n_dot_l, k);
        return ggx1 * ggx2;
    }

    fn distribution_ggx(&self, n: Vector3<f32>, h: Vector3<f32>) -> f32 {
        let a = self.roughness * self.roughness;
        let a2 = a * a;
        let n_dot_h = n.dot(h).max(0.);
        let n_dot_h_2 = n_dot_h * n_dot_h;

        let num = a2;
        let mut denom = n_dot_h_2 * (a2 - 1.) + 1.;
        denom = PI * denom * denom;

        return num / denom;
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

    fn compute_f0(metalness: f32, albedo: Vector3<f32>) -> Vector3<f32> {
        Vector3::new(0.04, 0.04, 0.04) * (1. - metalness) + albedo * metalness
    }
}
