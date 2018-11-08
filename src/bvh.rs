use bounding_box::BoundingBox;
use vec4d::Vec4d;
use intersectable::Intersectable;
use std::cmp::Ordering;
use collision::Collision;
use ray::Ray;

#[derive(Debug)]
enum BVHNode {
    Leaf((BoundingBox, Vec<usize>)),
    Node((BoundingBox, usize, Box<BVHNode>, Box<BVHNode>))
}

#[derive(Debug)]
pub struct BVH {
    root: BVHNode
}

fn tree_depth(node: &BVHNode) -> usize {
    match node {
        BVHNode::Leaf(_) => 1,
        BVHNode::Node((_, _, left, right)) => 1 + tree_depth(left).max(tree_depth(right))
    }
}

fn print_tree(node: &BVHNode) -> String {
    match node {
        BVHNode::Leaf((_, elements)) =>  format!("({:?})", elements),
        BVHNode::Node((_, _, left, right)) => format!("({}, {})", print_tree(left), print_tree(right))
    }
}

fn validate_tree<T:Intersectable>(node: &BVHNode, primitives: &[T]) -> BoundingBox {
    match node {
        BVHNode::Leaf((bounds, indices)) => {
            for index in indices.iter() {
                let primitive_bounds = primitives[*index].bounds();
                assert!(bounds.encloses(primitive_bounds))
            }
            return *bounds;
        },
        BVHNode::Node((bounds, _, left, right)) => {
            let left_bounds = validate_tree(left, primitives);
            let right_bounds = validate_tree(right, primitives);
            assert!(bounds.encloses(left_bounds));
            assert!(bounds.encloses(right_bounds));
            return * bounds;
        }
    }
}

#[derive(Copy, Clone)]
struct BVHPrimitiveInfo {
    pub primitive_number: usize,
    pub bounds: BoundingBox,
    pub centroid: Vec4d
}

impl BVHPrimitiveInfo {
    pub fn new(primitive_number: usize, bounds: BoundingBox) -> BVHPrimitiveInfo {
        BVHPrimitiveInfo {
            primitive_number:primitive_number,
            bounds:bounds,
            centroid:bounds.centroid()
        }
    }
}

impl BVH {
    pub fn new<T: Intersectable>(elements: &[T]) -> BVH {
        let mut info : Vec<BVHPrimitiveInfo> = Vec::new();
        for i in 0..elements.len() {
            let element = &elements[i];
            let inner_bounds = element.bounds();
            info.push(BVHPrimitiveInfo::new(i, inner_bounds));
        }
        let root = recursive_build(0, &mut info);
        let max_depth = tree_depth(&root);
    
        println!("Done building, max depth: {}", max_depth);
        println!("{}", print_tree(&root));
        validate_tree(&root, elements);
        BVH {
            root: root
        }
    }

    pub fn intersect<T: Intersectable>(&self, elements: &[T], ray: Ray, min: f64, max: f64) -> Option<Collision> {
        validate_tree(&self.root, elements);
        return intersect(&self.root, elements, ray, min, max);
    }
}

fn intersect_primitives<T: Intersectable>(indices:&[usize], primitives: &[T], ray: Ray, min: f64, max: f64) -> Option<Collision> {
        let mut closest = max;
        let mut result: Option<Collision> = None;
        for index in indices {
            let element = &primitives[*index];
            match element.intersect(ray, closest) {
                None => continue,
                Some(collision) => {
                    closest = collision.distance;
                    result = Some(collision);
                }
            }
        }
        return result;
}

fn intersect<T: Intersectable>(node: &BVHNode, elements: &[T], ray: Ray, min: f64, max: f64) -> Option<Collision> {
    let dir_is_negative = [ray.direction.x < 0., ray.direction.y < 0., ray.direction.z < 0.];
    match &node {
        BVHNode::Leaf((bounds, children)) => {
            if false {
                return intersect_primitives(children, elements, ray, min, max);
            }
            match bounds.intersect(ray, min, max) {
                None => None,
                Some((min, max)) => {
                    return intersect_primitives(children, elements, ray, min, max+1.);
                }
            }
        },
        BVHNode::Node((bounds, axis, left, right)) => {
            match bounds.intersect(ray, min, max) {
                None => None,
                Some(_) => {
                    let first : &BVHNode;
                    let second : &BVHNode;
                    if dir_is_negative[*axis] {
                        first = right;
                        second = left;
                    } else {
                        first = left;
                        second = right;
                    }
                    let mut result : Option<Collision> = None;
                    let mut inner_max = max;
                    match intersect(first, elements, ray, min, inner_max) {
                        None => {  },
                        Some(collision) => {
                            inner_max = collision.distance;
                            result = Some(collision);
                        }
                    }
                    match intersect(second, elements, ray, min, inner_max) {
                        None => { return result; },
                        value => {
                            return value;
                        }
                    }
                }
            }
        }
    }
}

const NUM_BUCKETS : usize = 12;
const MAX_PRIMS_PER_NODE : usize = 8;

#[derive(Copy, Clone, Debug)]
struct BucketInfo {
    pub count: usize,
    pub bounds: BoundingBox
}

fn recursive_build(depth: usize,
            primitives: &mut [BVHPrimitiveInfo]) -> BVHNode {
    let mut bounds = BoundingBox::new();
    for primitive in primitives.iter() {
        bounds = bounds.merge_with_bbox(primitive.bounds);
    }
    let length = primitives.len();
    let make_leaf = |nodes:&[BVHPrimitiveInfo]| {
        let mut bounds = BoundingBox::new();
        for primitive in nodes {
            bounds = bounds.merge_with_bbox(primitive.bounds)
        }
        return BVHNode::Leaf((bounds, nodes.iter().map(|primitive|{ primitive.primitive_number }).collect()));
    };

    if length == 1 {
        return make_leaf(primitives);
    }

    let mut centroid_bounds = BoundingBox::new();
    for primitive in primitives.iter() {
        centroid_bounds = centroid_bounds.merge_with_point(primitive.centroid);
    }

    let max_axis = centroid_bounds.max_axis();
    if centroid_bounds.max[max_axis] == centroid_bounds.min[max_axis] {
        return make_leaf(primitives);
    }

    if length <= 4 {
        // return make_leaf(primitives);
        primitives.sort_by(|left, right| {
            let lv = left.centroid[max_axis];
            let rv = right.centroid[max_axis];
            if lv < rv { return Ordering::Less; }
            if rv < lv { return Ordering::Greater; }
            return Ordering::Equal;
        });
        let mut inner_bounds = BoundingBox::new();
        for primitive in primitives.iter() {
            inner_bounds = inner_bounds.merge_with_bbox(primitive.bounds);
        }
        let left_child = Box::new(make_leaf(&primitives[0..(length/2)]));
        let right_child = Box::new(make_leaf(&primitives[(length/2)..]));
        return BVHNode::Node((inner_bounds, max_axis, left_child, right_child));
    }

    let mut buckets = [BucketInfo{count: 0, bounds: BoundingBox::new()}; NUM_BUCKETS];

    for primitive in primitives.iter() {
        let b = ((NUM_BUCKETS as f64 * centroid_bounds.offset(primitive.centroid)[max_axis]) as usize).min(NUM_BUCKETS - 1);
        assert!(b < NUM_BUCKETS);
        buckets[b].count += 1;
        buckets[b].bounds = buckets[b].bounds.merge_with_bbox(primitive.bounds);
        assert!(buckets[b].bounds.is_valid());
    }

    let mut cost= [0. as f64;NUM_BUCKETS - 1];
    for i in 0..(NUM_BUCKETS - 1) {
        let mut b0 = BoundingBox::new();
        let mut b1 = BoundingBox::new();
        let mut count0 = 0;
        let mut count1 = 0;
        for j in 0..=i {
            b0 = b0.merge_with_bbox(buckets[j].bounds);
            count0 += buckets[j].count;
        }
        for j in (i+1)..NUM_BUCKETS {
            b1 = b1.merge_with_bbox(buckets[j].bounds);
            count1 += buckets[j].count;
        }
        let left_cost = count0 as f64 * b0.surface_area();
        let right_cost = count1 as f64 * b1.surface_area();
        cost[i] = 0.125 + (left_cost + right_cost) / bounds.surface_area();
    }

    let mut min_cost = cost[0];
    let mut split_bucket = 0;
    for i in 1..(NUM_BUCKETS - 1) {
        if cost[i] < min_cost {
            min_cost = cost[i];
            split_bucket = i;
        }
    }

    let leaf_cost = length;

    if length < MAX_PRIMS_PER_NODE && min_cost >= leaf_cost as f64 {
        return make_leaf(primitives);
    }

    let mut left_primitives : Vec<BVHPrimitiveInfo> = Vec::new();
    let mut right_primitives : Vec<BVHPrimitiveInfo> = Vec::new();
    let centroid_split = centroid_bounds.min[max_axis] + (centroid_bounds.max - centroid_bounds.min)[max_axis] * split_bucket as f64 / NUM_BUCKETS as f64;
    let mut inner_bounds = BoundingBox::new();
    for primitive in primitives.iter() {
        inner_bounds = inner_bounds.merge_with_bbox(primitive.bounds);
        if primitive.centroid[max_axis] <= centroid_split {
            left_primitives.push(*primitive);
        } else {
            right_primitives.push(*primitive);
        }
    }
    
    assert!(left_primitives.len() != 0);
    assert!(right_primitives.len() != 0);
    assert!(left_primitives.len() + right_primitives.len() == primitives.len());
    let left_child = Box::new(recursive_build(depth + 1, &mut left_primitives));
    let right_child = Box::new(recursive_build(depth + 1, &mut right_primitives));
    return BVHNode::Node((inner_bounds, max_axis, left_child, right_child));
}
