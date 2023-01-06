use super::Vertex;
use crate::{
    renderer::{Hit, Intersectable, Ray},
    scene::isf,
};
use cgmath::*;
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
        ))
    }
}

impl BoundingBox for Triangle {
    fn bounding_box(&self) -> AABB {
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
        [min, max]
    }
}

impl From<isf::Triangle> for Triangle {
    fn from(isf: isf::Triangle) -> Self {
        Self(isf.0.into(), isf.1.into(), isf.2.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use yaml_rust::yaml;
    use yaml_rust::YamlLoader;

    #[derive(Debug)]
    struct TriangleHit {
        dist: f32,
        u: f32,
        v: f32,
    }

    #[derive(Debug)]
    struct Test {
        pub ray: Ray,
        pub triangle: Triangle,
        pub hit: Option<TriangleHit>,
    }

    fn array_to_vector3(array: &yaml::Array) -> Vector3<f32> {
        Vector3::new(
            array[0].as_f64().unwrap() as f32,
            array[1].as_f64().unwrap() as f32,
            array[2].as_f64().unwrap() as f32,
        )
    }

    fn convert_yaml(yaml: &yaml::Yaml) -> Test {
        let yaml = yaml.as_hash().unwrap();
        let mut ray = Ray::default();
        let mut triangle = Triangle::default();
        triangle[1].tex_coords = Vector2::new(1., 0.);
        triangle[2].tex_coords = Vector2::new(0., 1.);
        let mut hit = None;

        for (key, value) in yaml {
            let key = key.as_str().unwrap();
            let value = value.as_hash().unwrap();
            match key {
                "hit" => {
                    let mut u = 0.;
                    let mut v = 0.;
                    let mut dist = 0.;
                    for (key, value) in value {
                        let key = key.as_str().unwrap();
                        match key {
                            "u" => u = value.as_f64().unwrap() as f32,
                            "v" => v = value.as_f64().unwrap() as f32,
                            _ => dist = value.as_f64().unwrap() as f32,
                        }
                    }
                    hit = Some(TriangleHit { dist, u, v });
                }
                "ray" => {
                    for (key, value) in value {
                        let key = key.as_str().unwrap();
                        match key {
                            "direction" => {
                                ray.direction = array_to_vector3(value.as_vec().unwrap())
                            }
                            _ => ray.origin = array_to_vector3(value.as_vec().unwrap()),
                        }
                    }
                }
                "triangle" => {
                    for (key, value) in value {
                        let key = key.as_str().unwrap();
                        match key {
                            "v0" => {
                                triangle[0].position = array_to_vector3(value.as_vec().unwrap())
                            }
                            "v1" => {
                                triangle[1].position = array_to_vector3(value.as_vec().unwrap())
                            }
                            _ => triangle[2].position = array_to_vector3(value.as_vec().unwrap()),
                        }
                    }
                }
                _ => panic!("Malformated yaml test"),
            };
        }

        Test { ray, triangle, hit }
    }

    fn unwrap_hit_tex_coords(hit: &Hit) -> Vector2<f32> {
        match hit {
            Hit::Triangle { tex_coords, .. } => *tex_coords,
            _ => panic!("Hit is not a triangle"),
        }
    }

    #[test]
    fn hit() {
        let home = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut home = PathBuf::from(home);
        home.push("tests/moller_trumbore/hit_tests.yml");
        let tests = &YamlLoader::load_from_str(
            &fs::read_to_string(&home).expect("Something went wrong reading hit_tests.yml"),
        )
        .expect("Something went wrong parsing hit_tests.yml");
        let tests = &tests[0].as_vec().unwrap();

        for test in tests.iter() {
            let test = convert_yaml(test);
            let hit = test.triangle.intersect(&test.ray);
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
        let home = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut home = PathBuf::from(home);
        home.push("tests/moller_trumbore/miss_tests.yml");
        let tests = &YamlLoader::load_from_str(
            &fs::read_to_string(&home).expect("Something went wrong reading miss_tests.yml"),
        )
        .expect("Something went wrong parsing miss_tests.yml");
        let tests = &tests[0].as_vec().unwrap();

        for test in tests.iter() {
            let test = convert_yaml(test);
            assert!(test.triangle.intersect(&test.ray).is_none());
        }
    }
}
