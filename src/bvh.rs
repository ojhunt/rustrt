use bounding_box::BoundingBox;
use collision::Collision;
use intersectable::Intersectable;
use ray::Ray;
use shader::Shadable;
use vectors::Vec4d;

#[derive(Debug)]
enum BVHNode {
  Leaf((BoundingBox, Vec<usize>)),
  Node((BoundingBox, usize, Box<BVHNode>, Box<BVHNode>)),
}

#[derive(Debug)]
pub struct BVH {
  root: BVHNode,
}

#[derive(Copy, Clone)]
struct BVHPrimitiveInfo {
  pub primitive_number: usize,
  pub bounds: BoundingBox,
  pub centroid: Vec4d,
}

impl BVHPrimitiveInfo {
  pub fn new(primitive_number: usize, bounds: BoundingBox) -> BVHPrimitiveInfo {
    BVHPrimitiveInfo {
      primitive_number: primitive_number,
      bounds: bounds,
      centroid: bounds.centroid(),
    }
  }
}

fn tree_depth(node: &BVHNode) -> usize {
  match node {
    BVHNode::Leaf(_) => 1,
    BVHNode::Node((_, _, left, right)) => 1 + tree_depth(left).max(tree_depth(right)),
  }
}

impl BVH {
  pub fn new<T: Intersectable>(elements: &[T]) -> BVH {
    let mut info: Vec<BVHPrimitiveInfo> = Vec::new();
    for i in 0..elements.len() {
      let element = &elements[i];
      let inner_bounds = element.bounds();
      info.push(BVHPrimitiveInfo::new(i, inner_bounds));
    }
    let root = recursive_build(0, &mut info);
    println!("Tree max depth: {}", tree_depth(&root));
    BVH { root: root }
  }

  pub fn intersect<'a, T: Intersectable>(
    &self,
    elements: &'a [T],
    ray: &Ray,
    min: f64,
    max: f64,
  ) -> Option<(Collision, &'a Shadable)> {
    return intersect(&self.root, elements, ray, min, max);
  }
}

fn intersect_primitives<'a, T: Intersectable + 'a>(
  indices: &[usize],
  primitives: &'a [T],
  ray: &Ray,
  min: f64,
  max: f64,
) -> Option<(Collision, &'a Shadable)> {
  let mut closest = max;
  let mut result: Option<(Collision, &'a Shadable)> = None;
  for index in indices {
    let element = &primitives[*index];
    match element.intersect(ray, min, closest) {
      None => continue,
      Some((collision, object)) => {
        if collision.distance < closest {
          closest = collision.distance;
          result = Some((collision, object));
        }
      }
    }
  }
  return result;
}

fn intersect<'a, T: Intersectable>(
  node: &BVHNode,
  elements: &'a [T],
  ray: &Ray,
  parent_min: f64,
  parent_max: f64,
) -> Option<(Collision, &'a Shadable)> {
  let mut stack: Vec<(&BVHNode, /*min*/ f64, /*max*/ f64)> = Vec::new();
  stack.push((node, parent_min, parent_max));
  let mut result: Option<(Collision, &'a Shadable)> = None;
  let mut nearest = parent_max;
  let mut primitive_count = 0;
  let mut node_count = 0;
  while let Some((value, node_min, node_max)) = stack.pop() {
    node_count += 1;

    let dir_is_negative = [ray.direction.x() < 0., ray.direction.y() < 0., ray.direction.z() < 0.];

    if node_min > nearest {
      continue;
    }

    let far_intersect = nearest.min(node_max);

    match &value {
      BVHNode::Leaf((bounds, children)) => {
        match bounds.intersect(ray, node_min - 0.01, nearest) {
          None => continue,
          Some((min, max)) => {
            primitive_count += children.len();
            match intersect_primitives(children, elements, ray, min.max(node_min), nearest.min(max)) {
              None => continue,
              Some((inner_collision, object)) => {
                if inner_collision.distance < nearest {
                  nearest = inner_collision.distance;
                  result = Some((inner_collision, object));
                }
              }
            };
          }
        };
      }
      BVHNode::Node((bounds, axis, left, right)) => {
        match bounds.intersect(ray, node_min, far_intersect) {
          None => continue,
          Some((child_min, child_max)) => {
            if dir_is_negative[*axis] {
              stack.push((right, child_min, child_max));
              stack.push((left, child_min, child_max));
            } else {
              stack.push((left, child_min, child_max));
              stack.push((right, child_min, child_max));
            }
          }
        };
      }
    }
  }

  if let Some((c, object)) = result {
    result = Some((
      (Collision {
        distance: c.distance,
        uv: c.uv,
        intersection_count: primitive_count,
        node_count: node_count,
      }),
      object,
    ));
  }
  return result;
}

const NUM_BUCKETS: usize = 64;
const MAX_PRIMS_PER_NODE: usize = 4;

#[derive(Copy, Clone, Debug)]
struct BucketInfo {
  pub count: usize,
  pub bounds: BoundingBox,
}

fn recursive_build(depth: usize, primitives: &mut [BVHPrimitiveInfo]) -> BVHNode {
  let mut bounds = BoundingBox::new();
  for primitive in primitives.iter() {
    bounds = bounds.merge_with_bbox(primitive.bounds);
  }
  let length = primitives.len();
  let make_leaf = |nodes: &[BVHPrimitiveInfo]| {
    let mut bounds = BoundingBox::new();
    for primitive in nodes {
      bounds = bounds.merge_with_bbox(primitive.bounds)
    }
    return BVHNode::Leaf((
      bounds,
      nodes.iter().map(|primitive| primitive.primitive_number).collect(),
    ));
  };

  if length == 1 {
    return make_leaf(primitives);
  }

  let mut centroid_bounds = BoundingBox::new();
  for primitive in primitives.iter() {
    centroid_bounds = centroid_bounds.merge_with_point(primitive.centroid);
  }

  let max_axis = centroid_bounds.max_axis();
  if centroid_bounds.max.data[max_axis] == centroid_bounds.min.data[max_axis] {
    return make_leaf(primitives);
  }

  let mut buckets = [BucketInfo {
    count: 0,
    bounds: BoundingBox::new(),
  }; NUM_BUCKETS];

  for primitive in primitives.iter() {
    let b =
      ((NUM_BUCKETS as f32 * centroid_bounds.offset(primitive.centroid).data[max_axis]) as usize).min(NUM_BUCKETS - 1);
    assert!(b < NUM_BUCKETS);
    buckets[b].count += 1;
    buckets[b].bounds = buckets[b].bounds.merge_with_bbox(primitive.bounds);
    assert!(buckets[b].bounds.is_valid());
  }

  let mut cost = [0. as f64; NUM_BUCKETS - 1];
  for i in 0..(NUM_BUCKETS - 1) {
    let mut b0 = BoundingBox::new();
    let mut b1 = BoundingBox::new();
    let mut count0 = 0;
    let mut count1 = 0;
    for j in 0..=i {
      b0 = b0.merge_with_bbox(buckets[j].bounds);
      count0 += buckets[j].count;
    }
    for j in (i + 1)..NUM_BUCKETS {
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

  let mut left_primitives: Vec<BVHPrimitiveInfo> = Vec::new();
  let mut right_primitives: Vec<BVHPrimitiveInfo> = Vec::new();
  let centroid_split = centroid_bounds.min.data[max_axis]
    + (centroid_bounds.max - centroid_bounds.min).data[max_axis] * split_bucket as f32 / NUM_BUCKETS as f32;
  let mut inner_bounds = BoundingBox::new();
  for primitive in primitives.iter() {
    inner_bounds = inner_bounds.merge_with_bbox(primitive.bounds);
    if primitive.centroid.data[max_axis] <= centroid_split {
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
