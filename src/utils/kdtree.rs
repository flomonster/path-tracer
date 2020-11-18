use crate::utils::*;

#[derive(Clone, Debug)]
pub struct KDtree<P>
where
    P: BoundingBox + Clone,
{
    root: KDtreeNode<P>,
    space: AABB,
}

impl<P> KDtree<P>
where
    P: BoundingBox + Clone,
{
    pub fn new(mut items: Vec<P>) -> Self {
        let mut space = (Vector3::max_value(), Vector3::min_value());
        let mut aabb_items = vec![];
        while let Some(i) = items.pop() {
            let bb = i.bounding_box();

            if bb.0.x < space.0.x {
                space.0.x = bb.0.x;
            } else if bb.1.x > space.1.x {
                space.1.x = bb.1.x;
            }
            if bb.0.y < space.0.y {
                space.0.y = bb.0.y;
            } else if bb.1.y > space.1.y {
                space.1.y = bb.1.y;
            }
            if bb.0.z < space.0.z {
                space.0.z = bb.0.z;
            } else if bb.1.z > space.1.z {
                space.1.z = bb.1.z;
            }

            aabb_items.push((bb, i));
        }
        KDtree {
            space,
            root: KDtreeNode::new(space, aabb_items, 100),
        }
    }
}

#[derive(Clone, Debug)]
enum KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    Leaf {
        space: AABB,
        items: Vec<P>,
    },
    Node {
        left_space: AABB,
        left_node: Box<KDtreeNode<P>>,
        right_space: AABB,
        right_node: Box<KDtreeNode<P>>,
    },
}

enum Plan {
    X(f32),
    Y(f32),
    Z(f32),
}

impl<P> KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    fn find_plane(space: &AABB, _items: &Vec<(AABB, P)>, max_depth: usize) -> Plan {
        match max_depth % 3 {
            0 => Plan::X((space.0.x + space.1.x) / 2.),
            1 => Plan::Y((space.0.y + space.1.y) / 2.),
            _ => Plan::Z((space.0.z + space.1.z) / 2.),
        }
    }

    fn split_space(space: &AABB, plan: &Plan) -> (AABB, AABB) {
        let mut left = space.clone();
        let mut right = space.clone();
        match plan {
            Plan::X(x) => {
                left.0.x = *x;
                right.1.x = *x
            }
            Plan::Y(y) => {
                left.0.y = *y;
                right.1.y = *y
            }
            Plan::Z(z) => {
                left.0.z = *z;
                right.1.z = *z;
            }
        }
        (left, right)
    }

    fn intersect(a: &AABB, b: &AABB) -> bool {
        (a.0.x < b.1.x && a.1.x > b.0.x)
            && (a.0.y < b.1.y && a.1.y > b.0.y)
            && (a.0.z < b.1.z && a.1.z > b.0.z)
    }

    fn new(space: AABB, mut items: Vec<(AABB, P)>, max_depth: usize) -> Self {
        if items.len() <= 10 || max_depth == 0 {
            let mut res = vec![];
            while let Some(i) = items.pop() {
                res.push(i.1);
            }
            return Self::Leaf { space, items: res };
        }

        let p = Self::find_plane(&space, &items, max_depth);
        let (left_space, right_space) = Self::split_space(&space, &p);
        let left_items: Vec<(AABB, P)> = items
            .iter()
            .filter(|item| Self::intersect(&left_space, &item.0))
            .cloned()
            .collect();
        let right_items: Vec<(AABB, P)> = items
            .iter()
            .filter(|item| Self::intersect(&right_space, &item.0))
            .cloned()
            .collect();
        if max_depth > 2 && (left_items.len() == items.len() || right_items.len() == items.len()) {
            return Self::new(space, items, 2);
        }
        Self::Node {
            left_space,
            right_space,
            left_node: Box::new(Self::new(left_space, left_items, max_depth - 1)),
            right_node: Box::new(Self::new(right_space, right_items, max_depth - 1)),
        }
    }
}

impl<P> Intersectable<Vec<P>> for KDtree<P>
where
    P: BoundingBox + Clone,
{
    fn intersect(&self, ray: &Ray) -> Vec<P> {
        match &self.root {
            KDtreeNode::Leaf { space, items } => {
                if space.intersect(ray) {
                    items.clone()
                } else {
                    vec![]
                }
            }
            KDtreeNode::Node { .. } => self.root.intersect(ray),
        }
    }
}

impl<P> Intersectable<Vec<P>> for KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    fn intersect(&self, ray: &Ray) -> Vec<P> {
        match self {
            Self::Leaf { items, .. } => items.clone(),
            Self::Node {
                left_space,
                left_node,
                right_space,
                right_node,
            } => {
                let mut res = vec![];
                if right_space.intersect(ray) {
                    res = right_node.intersect(ray);
                }
                if left_space.intersect(ray) {
                    if res.is_empty() {
                        res = left_node.intersect(ray);
                    } else {
                        res.append(&mut left_node.intersect(ray));
                    }
                }
                res
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easy_gltf::model::Triangle;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use yaml_rust::yaml;
    use yaml_rust::YamlLoader;

    #[derive(Debug)]
    struct Test {
        pub ray: Ray,
        pub triangle: Triangle,
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

        for (key, value) in yaml {
            let key = key.as_str().unwrap();
            let value = value.as_hash().unwrap();
            match key {
                "hit" => {}
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

        Test { ray, triangle }
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

        let mut triangles = vec![];
        let mut rays = vec![];
        for test in tests.iter() {
            let test = convert_yaml(test);
            triangles.push(test.triangle);
            rays.push(test.ray);
        }
        let kdtree = KDtree::new(triangles);
        for r in rays.iter() {
            let res = kdtree.intersect(&r);
            assert!(!res.is_empty());
        }
    }
}
