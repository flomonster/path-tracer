use crate::utils::*;
use std::sync::Arc;

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

            aabb_items.push((bb, Arc::new(i)));
        }
        KDtree {
            space,
            root: KDtreeNode::new(space, aabb_items),
        }
    }
}

impl<P> BoundingBox for KDtree<P>
where
    P: BoundingBox + Clone,
{
    fn bounding_box(&self) -> AABB {
        self.space
    }
}

#[derive(Clone, Debug)]
enum KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    Leaf {
        space: AABB,
        items: Vec<Arc<P>>,
    },
    Node {
        left_space: AABB,
        left_node: Box<KDtreeNode<P>>,
        right_space: AABB,
        right_node: Box<KDtreeNode<P>>,
    },
}

#[derive(Clone)]
enum Plan {
    X(f32),
    Y(f32),
    Z(f32),
}

impl<P> KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    /// Compute volume of a box
    fn volume(v: &AABB) -> f32 {
        ((v.0.x - v.1.x) * (v.0.y - v.1.y) * (v.0.z - v.1.z)).abs()
    }

    /// Compute cost of a split (Kt = 15 and Ki = 20)
    fn cost(pl: f32, pr: f32, n_l: usize, n_r: usize) -> f32 {
        // Decrease cost if it cuts empty space
        let factor = if n_l == 0 || n_r == 0 { 0.8 } else { 1. };
        factor * (15. + 20. * (pl * n_l as f32 + pr * n_r as f32))
    }

    /// Surface Area Heuristic (SAH)
    fn sah(p: &Plan, v: &AABB, n_l: usize, n_r: usize) -> f32 {
        let (v_l, v_r) = Self::split_space(v, p);
        let vol_v = Self::volume(v);
        let pl = Self::volume(&v_l) / vol_v;
        let pr = Self::volume(&v_r) / vol_v;
        Self::cost(pl, pr, n_l, n_r)
    }

    fn classify(
        triangles: &Vec<(AABB, Arc<P>)>,
        v_l: &AABB,
        v_r: &AABB,
    ) -> (Vec<(AABB, Arc<P>)>, Vec<(AABB, Arc<P>)>) {
        let t_l: Vec<(AABB, Arc<P>)> = triangles
            .iter()
            .filter(|item| Self::intersect(v_l, &item.0))
            .cloned()
            .collect();
        let t_r: Vec<(AABB, Arc<P>)> = triangles
            .iter()
            .filter(|item| Self::intersect(v_r, &item.0))
            .cloned()
            .collect();
        (t_l, t_r)
    }

    fn perfect_splits(t: &AABB, v: &AABB) -> Vec<Plan> {
        let mut res = vec![];
        if t.0.x > v.0.x {
            res.push(Plan::X(t.0.x));
        }
        if t.0.y > v.0.y {
            res.push(Plan::Y(t.0.y));
        }
        if t.0.z > v.0.z {
            res.push(Plan::Z(t.0.z));
        }
        if t.1.x < v.1.x {
            res.push(Plan::X(t.1.x));
        }
        if t.1.y < v.1.y {
            res.push(Plan::Y(t.1.y));
        }
        if t.1.z < v.1.z {
            res.push(Plan::Z(t.1.z));
        }
        res
    }

    /// Compute best plan and it's cost
    fn partition(triangles: &Vec<(AABB, Arc<P>)>, v: &AABB) -> (f32, Plan) {
        let mut best_cost = f32::INFINITY;
        let mut best_plan = Plan::X(0.);
        for t in triangles {
            for p in Self::perfect_splits(&t.0, v).iter() {
                let (v_l, v_r) = Self::split_space(v, p);
                let (t_l, t_r) = Self::classify(triangles, &v_l, &v_r);
                let cost = Self::sah(p, v, t_l.len(), t_r.len());
                if cost < best_cost {
                    best_cost = cost;
                    best_plan = p.clone();
                }
            }
        }
        (best_cost, best_plan)
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

    fn new(space: AABB, mut items: Vec<(AABB, Arc<P>)>) -> Self {
        // Find best plan and his cost
        let (cost, p) = Self::partition(&items, &space);

        // If the cost of split is higher of cost of the current node then make a leaf
        if cost > 20. * items.len() as f32 {
            let mut res = vec![];
            while let Some(i) = items.pop() {
                res.push(i.1);
            }
            return Self::Leaf { space, items: res };
        }

        // Otherwise make a node
        let (v_l, v_r) = Self::split_space(&space, &p);
        let (t_l, t_r) = Self::classify(&items, &v_l, &v_r);

        Self::Node {
            left_space: v_l,
            right_space: v_r,
            left_node: Box::new(Self::new(v_l, t_l)),
            right_node: Box::new(Self::new(v_r, t_r)),
        }
    }
}

impl<P> Intersectable<Vec<Arc<P>>> for KDtree<P>
where
    P: BoundingBox + Clone,
{
    fn intersect(&self, ray: &Ray) -> Vec<Arc<P>> {
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

impl<P> Intersectable<Vec<Arc<P>>> for KDtreeNode<P>
where
    P: BoundingBox + Clone,
{
    fn intersect(&self, ray: &Ray) -> Vec<Arc<P>> {
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
