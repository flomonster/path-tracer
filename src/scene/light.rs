use cgmath::*;

pub enum Light {
    /// Directional light has a direction, color and intensity
    Directional(Vector3<f32>, Vector3<f32>, f32),

    /// Point light has a position, color and intensity
    Point(Vector3<f32>, Vector3<f32>, f32),
}
