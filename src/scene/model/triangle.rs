use super::Vertex;
use crate::utils::{Hit, Intersectable, Ray};
use cgmath::{InnerSpace, Vector2};

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // -----------------
        //  MOLLER TRUMBORE
        // -----------------

        let v0v1 = self.1.position - self.0.position;
        let v0v2 = self.2.position - self.0.position;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        // Check face culling
        if det < 0.0001 {
            return None;
        }

        let invdet = 1. / det;

        let tvec = ray.origin - self.0.position;
        let u = tvec.dot(pvec) * invdet;
        if u < 0. || u > 1. {
            return None;
        }

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * invdet;
        if v < 0. || u + v > 1. {
            return None;
        }

        let dist = v0v2.dot(qvec) * invdet;

        // Check triangle behind
        if dist < 0.0001 {
            return None;
        }

        Some(Hit {
            dist,
            triangle: (*self).clone(),
            position: ray.origin + ray.direction * dist,
            uv: Vector2::new(u, v),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::*;

    #[test]
    fn not_intersected() {
        let template_vertex = Vertex::new(0., 0., 0., 0., 0., 0., 0., 0.);
        let ray = Ray::new(Vector3::new(-0.2, 0., 2.), Vector3::new(0., 0., -1.));
        let triangle = Triangle(
            Vertex {
                position: Vector3::new(0., 0., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(1., 0., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(1., 1., 0.),
                ..template_vertex
            },
        );
        assert_eq!(triangle.intersect(&ray), None);
    }

    #[test]
    fn intersected() {
        let template_vertex = Vertex::new(0., 0., 0., 0., 0., 0., 0., 0.);
        let ray = Ray::new(Vector3::new(0., 0., 2.), Vector3::new(0., 0., -1.));
        let triangle = Triangle(
            Vertex {
                position: Vector3::new(-1., -1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(1., -1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(-1., 1., 0.),
                ..template_vertex
            },
        );
        assert_eq!(triangle.intersect(&ray).unwrap().dist, 2.);
    }

    #[test]
    fn behind() {
        let template_vertex = Vertex::new(0., 0., 0., 0., 0., 0., 0., 0.);
        let ray = Ray::new(Vector3::new(0., 0., -2.), Vector3::new(0., 0., -1.));
        let triangle = Triangle(
            Vertex {
                position: Vector3::new(-1., -1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(1., -1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(-1., 1., 0.),
                ..template_vertex
            },
        );
        assert!(triangle.intersect(&ray).is_none());
    }

    #[test]
    fn backface() {
        let template_vertex = Vertex::new(0., 0., 0., 0., 0., 0., 0., 0.);
        let ray = Ray::new(Vector3::new(0., 0., 2.), Vector3::new(0., 0., -1.));
        let triangle = Triangle(
            Vertex {
                position: Vector3::new(-1., -1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(-1., 1., 0.),
                ..template_vertex
            },
            Vertex {
                position: Vector3::new(1., -1., 0.),
                ..template_vertex
            },
        );
        assert_eq!(triangle.intersect(&ray), None);
    }
}
