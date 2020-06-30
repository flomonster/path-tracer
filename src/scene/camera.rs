use cgmath::Rad;
use cgmath::*;

pub struct Camera {
    /// Position of the camera
    pub position: Vector3<f32>,

    /// Right vector of the camera
    pub right: Vector3<f32>,

    /// Up vector of the camera
    pub up: Vector3<f32>,

    /// Forward vector of the camera (backside direction)
    pub forward: Vector3<f32>,

    /// Angle in degree of field of view
    pub fov: Rad<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vector3::new(0., 0., 2.),
            right: Vector3::new(1., 0., 0.),
            up: Vector3::new(0., 1., 0.),
            forward: Vector3::new(0., 0., -1.),
            fov: Rad(90.0),
        }
    }
}
