use super::transform_to_world;
use crate::renderer::brdf::Brdf;
use crate::renderer::MaterialSample;
use cgmath::*;
use std::f32::consts::PI;

pub struct CookTorrance {
    metalness: f32,
    roughness: f32,
    albedo: Vector3<f32>,
    ambient_occlusion: Vector3<f32>,
    f0: Vector3<f32>, // surface reflection at zero incidence from Fresnel-Schlick approximation
    microfacet_normal: Vector3<f32>, // a.k.a. Wm
}

impl Default for CookTorrance {
    fn default() -> Self {
        CookTorrance {
            metalness: 0.,
            roughness: 0.,
            albedo: Zero::zero(),
            ambient_occlusion: Zero::zero(),
            f0: Zero::zero(),
            microfacet_normal: Zero::zero(),
        }
    }
}

impl Brdf for CookTorrance {
    fn new(material: &MaterialSample, geometric_normal: Vector3<f32>) -> Self {
        Self {
            metalness: material.metalness,
            roughness: material.roughness,
            albedo: material.albedo,
            ambient_occlusion: Self::compute_ambient_occlusion(
                material.albedo,
                material.ambient_occlusion,
            ),
            f0: Self::compute_f0(material.metalness, material.albedo),
            microfacet_normal: Self::brdf_get_microfacet_normal(
                geometric_normal,
                material.roughness,
            ),
        }
    }

    fn sample(&self, v: Vector3<f32>) -> Vector3<f32> {
        // Compute direction by reflecting v about the microfacet normal
        let sample_dir = 2. * v.dot(self.microfacet_normal) * self.microfacet_normal - v;
        // TODO: try using 'reflection' function from utils
        return sample_dir.normalize();
    }

    // Cook-Torrance specular BRDF
    fn eval(
        &self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>,  // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
        light_radiance: Vector3<f32>,
    ) -> Vector3<f32> {
        let n = geometric_normal;
        let v = view_direction;
        let l = light_direction;

        let halfway = (v + l).normalize();

        // TODO: try using microfacet_normal instead of geometric_normal
        let d = self.distribution_ggx(n, halfway);
        let f = self.fresnel_schlick(halfway.dot(v).max(0.));
        let g = self.geometry_smith(n, v, l);

        // Specular
        let specular = (d * f * g) / (4. * n.dot(v).max(0.) * n.dot(l).max(0.)).max(0.001);

        // Diffuse
        let kd = Vector3::new(1. - f.x, 1. - f.y, 1. - f.z) * (1. - self.metalness);
        let diffuse = kd.mul_element_wise(self.albedo) / PI;

        return (diffuse + specular).mul_element_wise(light_radiance) * n.dot(l).max(0.);
    }

    fn pdf(&self, geometric_normal: Vector3<f32>, v: Vector3<f32>, l: Vector3<f32>) -> f32 {
        // Use NDF of the Cook-Torrance Microfacet model as the PDF
        let halfway = (v + l).normalize();
        let ndf = self.distribution_ggx(self.microfacet_normal, halfway); // TODO: try using geometric_normal instead of halfway
        let weight =
            self.microfacet_normal.dot(geometric_normal) / (4. * v.dot(self.microfacet_normal));
        let pdf = ndf * weight;
        return pdf;

        // TODO: Note: We could simplify the BRDF by canceling the NDF term
    }

    fn get_ambient_occlusion(&self) -> Vector3<f32> {
        self.ambient_occlusion
    }
}

impl CookTorrance {
    fn brdf_get_microfacet_normal(geometric_normal: Vector3<f32>, roughness: f32) -> Vector3<f32> {
        let a = roughness * roughness;
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

        return (transform_to_world(geometric_normal) * microfacet_normal).normalize();
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
        let a = self.roughness; // * self.roughness;
        let k = (a + 1.).powi(2) / 8.;
        let n_dot_v = n.dot(v).max(0.);
        let n_dot_l = n.dot(l).max(0.);
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
