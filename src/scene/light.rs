use cgmath::*;

pub enum Light {
    /// Directional light has a direction, color and intensity
    Directional(Vector3<f32>, Vector3<f32>, f32),

    /// Point light has a position, color and intensity
    Point(Vector3<f32>, Vector3<f32>, f32),
}

impl Light {
    pub fn new_directional(direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Light::Directional(direction.normalize(), color, intensity)
    }
    pub fn new_point(position: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Light::Point(position, color, intensity)
    }
}
