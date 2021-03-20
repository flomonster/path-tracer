mod cook_torrance;

use easy_gltf::Material;
use std::sync::Arc;
use cgmath::*;
pub use cook_torrance::CookTorrance;

pub trait Brdf: Default {
    fn new(material: &Arc<Material>, tex_coords: Vector2<f32>, geometric_normal: Vector3<f32>) -> Self;

    fn sample(&self, v: Vector3<f32>) -> Vector3<f32>;

    fn eval(&self,
        geometric_normal: Vector3<f32>,
        view_direction: Vector3<f32>, // from hit point to the viewer
        light_direction: Vector3<f32>, // from hit point to the light
        light_radiance: Vector3<f32>    
    ) -> Vector3<f32>;

    fn pdf(&self, geometric_normal: Vector3<f32>, v: Vector3<f32>, l: Vector3<f32>) -> f32;

    fn get_ambient_occlusion(&self) -> Vector3<f32>; 
}

// Transform any coordinate system to world coordinates
pub fn transform_to_world(n: Vector3<f32>) -> Matrix3<f32> {
    // Create a local coordinate system (n, nb, nt) oriented along the normal
    let nt = if n.x.abs() > n.y.abs() {
        Vector3::new(n.z, 0., -n.x) / (n.x * n.x + n.z * n.z).sqrt()
    } else {
        Vector3::new(0., -n.z, n.y) / (n.y * n.y + n.z * n.z).sqrt()
    };
    let nb = n.cross(nt);

    // Return the transformation matrix
    return Matrix3::new(nb.x, n.x, nt.x, nb.y, n.y, nt.y, nb.z, n.z, nt.z);
}