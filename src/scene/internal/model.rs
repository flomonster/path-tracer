use super::texture_bank::TextureBank;
use super::{Material, Triangle};
use crate::renderer::{Hit, Intersectable, Ray};
use crate::scene::isf;
use cgmath::InnerSpace;
use cgmath::Vector3;
use kdtree_ray::{BoundingBox, KDtree, AABB};

#[derive(Clone, Debug)]
pub enum Model {
    Mesh {
        triangles: KDtree<Triangle>,
        material: Material,
    },
    Sphere {
        radius: f32,
        center: Vector3<f32>,
        material: Material,
    },
}

impl Intersectable<Vec<Hit>> for Model {
    fn intersect(&self, ray: &Ray) -> Vec<Hit> {
        match self {
            Model::Sphere { radius, center, .. } => {
                let ray_to_center = ray.origin - center;
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * ray_to_center.dot(ray.direction);
                let c = ray_to_center.dot(ray_to_center) - radius * radius;
                let discriminant = b * b - 4.0 * a * c;
                if discriminant < 0.0 {
                    return vec![];
                }
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                assert!(t1 <= t2);
                // Doesn't authorize intersections from inside the sphere
                if t1 < 0.0 {
                    return vec![];
                }
                let hit_point = ray.origin + ray.direction * t1;
                let normal = (hit_point - center).normalize();
                vec![Hit::Sphere {
                    dist: ray_to_center.magnitude(),
                    position: hit_point,
                    normal,
                }]
            }
            Model::Mesh { triangles, .. } => triangles
                .intersect(&ray.origin, &ray.direction)
                .iter()
                .filter_map(|t| t.intersect(ray))
                .collect(),
        }
    }
}

impl BoundingBox for Model {
    fn bounding_box(&self) -> AABB {
        match self {
            Model::Mesh { triangles, .. } => triangles.bounding_box(),
            Model::Sphere { radius, center, .. } => [
                *center - Vector3::new(*radius, *radius, *radius),
                *center + Vector3::new(*radius, *radius, *radius),
            ],
        }
    }
}

impl Model {
    pub fn load(isf: isf::Model, texture_bank: &mut TextureBank) -> Self {
        match isf {
            isf::Model::Mesh {
                triangles,
                material,
            } => Model::Mesh {
                triangles: KDtree::new(triangles.iter().map(|t| t.clone().into()).collect()),
                material: Material::load(material, texture_bank),
            },
            isf::Model::Sphere {
                radius,
                center,
                material,
            } => Model::Sphere {
                radius,
                center: center.into(),
                material: Material::load(material, texture_bank),
            },
        }
    }

    pub fn get_material(&self) -> &Material {
        match self {
            Model::Mesh { material, .. } => material,
            Model::Sphere { material, .. } => material,
        }
    }
}
