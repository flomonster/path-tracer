use cgmath::{Matrix4, Rad, Vector3, Vector4};

use crate::scene::isf;

/// Contains camera properties.
#[derive(Clone, Debug)]
pub struct Camera {
    /// Transform matrix (also called world to camera matrix)
    pub transform: Matrix4<f32>,

    /// Angle in degree of field of view
    pub fov: Rad<f32>,

    /// The distance to the far clipping plane.
    #[allow(dead_code)]
    pub zfar: f32,

    /// The distance to the near clipping plane.
    #[allow(dead_code)]
    pub znear: f32,
}

impl From<isf::Camera> for Camera {
    fn from(c: isf::Camera) -> Self {
        Camera {
            transform: c.transform.into(),
            fov: Rad(c.fov),
            zfar: c.zfar,
            znear: c.znear,
        }
    }
}

impl Camera {
    /// Apply the transformation matrix on a vector
    pub fn apply_transform_vector(&self, pos: &Vector3<f32>) -> Vector3<f32> {
        let pos = Vector4::new(pos.x, pos.y, pos.z, 0.0);
        (self.transform * pos).truncate()
    }

    /// Position of the camera
    pub fn position(&self) -> Vector3<f32> {
        Vector3::new(
            self.transform[3][0],
            self.transform[3][1],
            self.transform[3][2],
        )
    }
}
