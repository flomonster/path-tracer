use super::texture_bank::TextureBank;
use super::{Material, Triangle};
use crate::renderer::{Hit, Intersectable, Ray};
use crate::scene::isf;
use cgmath::InnerSpace;
use cgmath::Vector3;
use kdtree_ray::{Bounded, KDTree, AABB};

#[derive(Clone, Debug)]
pub enum Model {
    Mesh {
        triangles: Vec<Triangle>,
        kdtree: KDTree,
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
                if t2 < 0.0 {
                    // Sphere is behind us
                    return vec![];
                }

                let hit_point = ray.origin + ray.direction * t2;
                let normal = -(hit_point - center).normalize();
                let hit_t2 = Hit::Sphere {
                    dist: (hit_point - ray.origin).magnitude(),
                    position: hit_point,
                    normal,
                };
                if t1 < 0.0 {
                    // We are inside the sphere
                    vec![hit_t2]
                } else {
                    // Both intersections are in front of us
                    let hit_point = ray.origin + ray.direction * t1;
                    let normal = (hit_point - center).normalize();
                    let hit_t1 = Hit::Sphere {
                        dist: (hit_point - ray.origin).magnitude(),
                        position: hit_point,
                        normal,
                    };
                    vec![hit_t1, hit_t2]
                }
            }
            Model::Mesh {
                triangles, kdtree, ..
            } => kdtree
                .intersect(&ray.origin, &ray.direction)
                .into_iter()
                .filter_map(|index| triangles[index].intersect(ray))
                .collect(),
        }
    }
}

impl Bounded for Model {
    fn bound(&self) -> AABB {
        match self {
            Model::Mesh { kdtree, .. } => kdtree.bound(),
            Model::Sphere { radius, center, .. } => AABB::new(
                *center - Vector3::new(*radius, *radius, *radius),
                *center + Vector3::new(*radius, *radius, *radius),
            ),
        }
    }
}

impl Model {
    pub fn load(isf: isf::Model, texture_bank: &mut TextureBank) -> Self {
        match isf {
            isf::Model::Mesh {
                triangles,
                material,
            } => {
                let triangles = triangles.into_iter().map(|t| t.into()).collect();
                let kdtree = KDTree::build(&triangles);
                Model::Mesh {
                    triangles,
                    kdtree,
                    material: Material::load(material, texture_bank),
                }
            }
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
