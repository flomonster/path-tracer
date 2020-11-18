use cgmath::*;

#[derive(Debug)]
pub struct Ray {
    /// Origin of the ray
    pub origin: Vector3<f32>,

    /// Direction of the ray
    pub direction: Vector3<f32>,
}

impl Ray {
    /// Create a Ray given origin and direction
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        Ray { origin, direction }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            origin: Vector3::zero(),
            direction: Vector3::zero(),
        }
    }
}

pub trait Intersectable<P> {
    fn intersect(&self, ray: &Ray) -> P;
}

/// Axis-aligned bounding boxes
pub type AABB = (Vector3<f32>, Vector3<f32>);

impl BoundingBox for AABB {
    fn bounding_box(&self) -> AABB {
        self.clone()
    }
}

pub trait BoundingBox {
    fn bounding_box(&self) -> AABB;
}

impl Intersectable<bool> for AABB {
    fn intersect(&self, ray: &Ray) -> bool {
        let mut tmin = (self.0.x - ray.origin.x) / ray.direction.x;
        let mut tmax = (self.1.x - ray.origin.x) / ray.direction.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.0.y - ray.origin.y) / ray.direction.y;
        let mut tymax = (self.1.y - ray.origin.y) / ray.direction.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return false;
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.0.z - ray.origin.z) / ray.direction.z;
        let mut tzmax = (self.1.z - ray.origin.z) / ray.direction.z;

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return false;
        }

        true
    }
}
