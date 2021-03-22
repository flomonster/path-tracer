mod cook_torrance;

use super::MaterialSample;
use cgmath::*;
pub use cook_torrance::CookTorrance;

pub trait Brdf: Default {
    fn new(material: &MaterialSample, geometric_normal: Vector3<f32>) -> Self;

    fn sample(&self, v: Vector3<f32>) -> Vector3<f32>;

    fn eval_direct(&self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>, // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
        light_radiance: Vector3<f32>    
    ) -> Vector3<f32>;

    fn eval_indirect(&self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>,  // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
        light_radiance: Vector3<f32>,
    ) -> Vector3<f32>;

    fn pdf(&self, geometric_normal: Vector3<f32>, v: Vector3<f32>, l: Vector3<f32>) -> f32;

    fn get_ambient_occlusion(&self) -> Vector3<f32>;
}

// Transform any coordinate system to world coordinates
pub fn transform_to_world(vec: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    // Find an axis that is not parallel to normal
    let magic_number = 1. / f32::sqrt(3.);
    let major_axis = if n.x.abs() < magic_number {
        Vector3::<f32>::new(1., 0., 0.)
    } else if n.y.abs() < magic_number {
        Vector3::<f32>::new(0., 1., 0.)
    } else {
        Vector3::<f32>::new(0., 0., 1.)
    };

    // Use majorAxis to create a coordinate system relative to world space
    let u = n.cross(major_axis).normalize();
    let v = n.cross(u);
    let w = n;

    // Transform from local coordinates to world coordinates
    return u * vec.x + v * vec.y + w * vec.z;
}
