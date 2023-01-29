use super::Vertex;
use crate::{
    renderer::{Hit, Intersectable, Ray},
    scene::isf,
};
use cgmath::{InnerSpace, Vector2, Vector3};
use kdtree_ray::*;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triangle(Vertex, Vertex, Vertex);

impl Index<usize> for Triangle {
    type Output = Vertex;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bound [0-2]"),
        }
    }
}

impl IndexMut<usize> for Triangle {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bound [0-2]"),
        }
    }
}

impl Intersectable<Option<Hit>> for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // -----------------
        //  MOLLER TRUMBORE
        // -----------------

        let v0v1 = self[1].position - self[0].position;
        let v0v2 = self[2].position - self[0].position;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        // Check parallel face (backface culling is disabled)
        if det.abs() < 0.000001 {
            return None;
        }

        let invdet = 1. / det;

        let tvec = ray.origin - self[0].position;
        let u = tvec.dot(pvec) * invdet;
        if !(0.0..=1.).contains(&u) {
            return None;
        }

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * invdet;
        if v < 0. || u + v > 1. {
            return None;
        }

        let dist = v0v2.dot(qvec) * invdet;

        // Check triangle behind
        if dist < 0.000001 {
            return None;
        }

        Some(Hit::new_triangle(
            (*self).clone(),
            dist,
            ray.origin + ray.direction * dist,
            &Vector2::new(u, v),
            det < 0.0,
        ))
    }
}

impl Bounded for Triangle {
    fn bound(&self) -> AABB {
        let min = Vector3::new(
            self[0]
                .position
                .x
                .min(self[1].position.x)
                .min(self[2].position.x),
            self[0]
                .position
                .y
                .min(self[1].position.y)
                .min(self[2].position.y),
            self[0]
                .position
                .z
                .min(self[1].position.z)
                .min(self[2].position.z),
        );
        let max = Vector3::new(
            self[0]
                .position
                .x
                .max(self[1].position.x)
                .max(self[2].position.x),
            self[0]
                .position
                .y
                .max(self[1].position.y)
                .max(self[2].position.y),
            self[0]
                .position
                .z
                .max(self[1].position.z)
                .max(self[2].position.z),
        );
        AABB::new(min, max)
    }
}

impl From<isf::Triangle> for Triangle {
    fn from(isf: isf::Triangle) -> Self {
        Self(isf.0.into(), isf.1.into(), isf.2.into())
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;

    use super::*;

    #[derive(Deserialize, Serialize, Debug)]
    struct HitTest {
        dist: f32,
        u: f32,
        v: f32,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RayTest {
        direction: [f32; 3],
        position: [f32; 3],
    }
    impl From<RayTest> for Ray {
        fn from(r: RayTest) -> Self {
            Self {
                direction: r.direction.into(),
                origin: r.position.into(),
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TriangleTest {
        v0: [f32; 3],
        v1: [f32; 3],
        v2: [f32; 3],
    }

    impl From<TriangleTest> for Triangle {
        fn from(t: TriangleTest) -> Self {
            Self(
                Vertex {
                    position: t.v0.into(),
                    ..Default::default()
                },
                Vertex {
                    position: t.v1.into(),
                    tex_coords: Vector2::new(1.0, 0.0),
                    ..Default::default()
                },
                Vertex {
                    position: t.v2.into(),
                    tex_coords: Vector2::new(0.0, 1.0),
                    ..Default::default()
                },
            )
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct Test {
        pub ray: RayTest,
        pub triangle: TriangleTest,
        pub hit: Option<HitTest>,
    }

    fn unwrap_hit_tex_coords(hit: &Hit) -> Vector2<f32> {
        match hit {
            Hit::Triangle { tex_coords, .. } => *tex_coords,
            _ => panic!("Hit is not a triangle"),
        }
    }

    #[test]
    fn hit() {
        let tests: Vec<Test> =
            serde_yaml::from_str(include_str!("../../../tests/moller_trumbore/hit_tests.yml"))
                .unwrap();

        for test in tests.into_iter() {
            let triangle: Triangle = test.triangle.into();
            let ray = test.ray.into();
            let hit = triangle.intersect(&ray);
            assert!(hit.is_some());
            let hit = hit.unwrap();
            let test_hit = test.hit.unwrap();
            assert!((hit.get_dist() - test_hit.dist).abs() < 0.00001);
            let tex_coords = unwrap_hit_tex_coords(&hit);
            assert!((tex_coords[0] - test_hit.u).abs() < 0.00001);
            assert!((tex_coords[1] - test_hit.v).abs() < 0.00001);
        }
    }

    #[test]
    fn miss() {
        let tests: Vec<Test> = serde_yaml::from_str(include_str!(
            "../../../tests/moller_trumbore/miss_tests.yml"
        ))
        .unwrap();

        for test in tests.into_iter() {
            let triangle: Triangle = test.triangle.into();
            let ray = test.ray.into();
            let hit = triangle.intersect(&ray);
            assert!(hit.is_none());
        }
    }
}
