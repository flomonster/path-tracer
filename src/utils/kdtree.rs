use crate::utils::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct KDtree<P>
where
    P: BoundingBox + Clone,
{
    root: KDtreeNode<P>,
    space: AABB,
}

type Items<P> = Vec<Item<P>>;

#[derive(Clone, Debug)]
struct Item<P>
where
    P: BoundingBox + Clone,
{
    pub value: Arc<P>,
    pub bb: AABB,
    pub id: usize,
}

impl<P> Item<P>
where
    P: BoundingBox + Clone,
{
    fn new(value: P, id: usize) -> Self {
        let bb = value.bounding_box();
        Item {
            value: Arc::new(value),
            bb,
            id,
        }
    }
}

impl<P> KDtree<P>
where
    P: BoundingBox + Clone,
{
    pub fn new(mut values: Vec<P>) -> Self {
        let mut space = (Vector3::max_value(), Vector3::min_value());
        let mut items = Items::new();
        let mut id = 0;
        while let Some(v) = values.pop() {
            let item = Item::new(v, id);
            id += 1;

            if item.bb.0.x < space.0.x {
                space.0.x = item.bb.0.x;
            }
            if item.bb.1.x > space.1.x {
                space.1.x = item.bb.1.x;
            }
            if item.bb.0.y < space.0.y {
                space.0.y = item.bb.0.y;
            }
            if item.bb.1.y > space.1.y {
                space.1.y = item.bb.1.y;
            }
            if item.bb.0.z < space.0.z {
                space.0.z = item.bb.0.z;
            }
            if item.bb.1.z > space.1.z {
                space.1.z = item.bb.1.z;
            }
            items.push(item);
        }
        let root = KDtreeNode::new(space, items);
        KDtree { space, root }
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

#[derive(Clone, Debug)]
enum Plan {
    X(f32),
    Y(f32),
    Z(f32),
}

impl Eq for Plan {}

impl Ord for Plan {
    fn cmp(&self, other: &Self) -> Ordering {
        let v = match self {
            Plan::X(v) => v,
            Plan::Y(v) => v,
            Plan::Z(v) => v,
        };
        let v_other = match other {
            Plan::X(v) => v,
            Plan::Y(v) => v,
            Plan::Z(v) => v,
        };
        if v < v_other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl PartialEq for Plan {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl PartialOrd for Plan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
        let (vol_l, vol_r) = (Self::volume(&v_l), Self::volume(&v_r));
        let vol_v = Self::volume(v);
        if vol_v == 0. || vol_l == 0. || vol_r == 0. {
            return f32::INFINITY;
        }
        let pl = vol_l / vol_v;
        let pr = vol_r / vol_v;
        Self::cost(pl, pr, n_l, n_r)
    }

    fn classify(
        event_list: &Vec<(Plan, bool, Item<P>)>,
        best_event: usize,
    ) -> (Items<P>, Items<P>) {
        let mut left_items = vec![];
        let mut right_items = vec![];
        let mut start_left = HashSet::new();
        for i in 0..best_event {
            if !event_list[i].1 {
                left_items.push(event_list[i].2.clone());
            } else {
                start_left.insert(event_list[i].2.id);
            }
        }
        for i in (1 + best_event)..event_list.len() {
            if event_list[i].1 {
                right_items.push(event_list[i].2.clone());
            } else if start_left.contains(&event_list[i].2.id) {
                left_items.push(event_list[i].2.clone());
                right_items.push(event_list[i].2.clone());
            }
        }
        if event_list[best_event].1 {
            right_items.push(event_list[best_event].2.clone());
        } else {
            left_items.push(event_list[best_event].2.clone());
        }
        (left_items, right_items)
    }

    fn perfect_splits(item: &Item<P>, v: &AABB, dim: usize) -> Vec<(Plan, bool, Item<P>)> {
        let mut res = vec![];
        match dim {
            0 => {
                res.push((Plan::X(item.bb.0.x.max(v.0.x)), true, item.clone()));
                res.push((Plan::X(item.bb.1.x.min(v.1.x)), false, item.clone()));
            }
            1 => {
                res.push((Plan::Y(item.bb.0.y.max(v.0.y)), true, item.clone()));
                res.push((Plan::Y(item.bb.1.y.min(v.1.y)), false, item.clone()));
            }
            2 => {
                res.push((Plan::Z(item.bb.0.z.max(v.0.z)), true, item.clone()));
                res.push((Plan::Z(item.bb.1.z.min(v.1.z)), false, item.clone()));
            }
            _ => panic!("Invalid dimension number received: ({})", dim),
        }
        res
    }

    /// Compute best plan and it's cost
    fn partition(items: &Items<P>, v: &AABB) -> (f32, usize, Vec<(Plan, bool, Item<P>)>) {
        let mut best_cost = f32::INFINITY;
        let mut best_plan = 0;
        let mut best_event_list = vec![];
        for dim in 0..3 {
            let mut event_list = vec![];
            for item in items {
                event_list.append(&mut Self::perfect_splits(&item, v, dim));
            }
            event_list.sort_by(|a, b| a.0.cmp(&b.0));
            let mut n_l = 0;
            let mut n_r = items.len();
            let mut i = 0;
            let mut best_changed = false;
            while i < event_list.len() {
                let current_plan = i;
                let mut p_true = 0;
                let mut p_false = 0;
                // Plan p is type true (+)
                if event_list[current_plan].1 {
                    while i < event_list.len() && event_list[i].1 {
                        i += 1;
                        p_true += 1;
                    }
                }
                // Plan p is type false (-)
                else {
                    while i < event_list.len() && !event_list[i].1 {
                        i += 1;
                        p_false += 1;
                    }
                }
                n_r -= p_false;
                let cost = Self::sah(&event_list[current_plan].0, v, n_l, n_r);
                if cost < best_cost {
                    best_cost = cost;
                    best_plan = current_plan;
                    best_changed = true;
                }
                n_l += p_true;
            }
            if best_changed {
                best_event_list = event_list.clone();
            }
        }
        (best_cost, best_plan, best_event_list)
    }

    fn split_space(space: &AABB, plan: &Plan) -> (AABB, AABB) {
        let mut left = space.clone();
        let mut right = space.clone();
        match plan {
            Plan::X(x) => {
                left.1.x = *x;
                right.0.x = *x
            }
            Plan::Y(y) => {
                left.1.y = *y;
                right.0.y = *y
            }
            Plan::Z(z) => {
                left.1.z = *z;
                right.0.z = *z;
            }
        }
        (left, right)
    }

    fn new(space: AABB, mut items: Items<P>) -> Self {
        // Find best plan and his cost
        let (cost, best_event, event_list) = Self::partition(&items, &space);

        // If the cost of split is higher of cost of the current node then make a leaf
        if cost > 20. * items.len() as f32 {
            let mut res = vec![];
            while let Some(i) = items.pop() {
                res.push(i.value);
            }
            return Self::Leaf { space, items: res };
        }

        // Otherwise make a node
        let (v_l, v_r) = Self::split_space(&space, &event_list[best_event].0);
        let (t_l, t_r) = Self::classify(&event_list, best_event);

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
