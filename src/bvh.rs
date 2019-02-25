use crate::bounding_box::BoundingBox;
use crate::collision::Collision;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::shader::Shadable;
use crate::vectors::{Point, VectorType};

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
  pub centroid: Point,
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

impl BVH {
  pub fn new<T: Intersectable>(elements: &[T]) -> BVH {
    let mut info: Vec<BVHPrimitiveInfo> = Vec::new();
    for i in 0..elements.len() {
      let element = &elements[i];
      let inner_bounds = element.bounds();
      info.push(BVHPrimitiveInfo::new(i, inner_bounds));
    }
    let root = recursive_build(0, &mut info);
    BVH { root: root }
  }

  pub fn intersect<'a, T: Intersectable>(
    &self,
    elements: &'a [T],
    ray: &Ray,
    first_hit_only: bool,
    min: f32,
    max: f32,
  ) -> Option<(Collision, &'a Shadable)> {
    return intersect(&self.root, elements, ray, first_hit_only, min, max);
  }
}

fn intersect_primitives<'a, T: Intersectable + 'a>(
  indices: &[usize],
  primitives: &'a [T],
  ray: &Ray,
  first_hit_only: bool,
  min: f32,
  max: f32,
) -> Option<(Collision, &'a Shadable)> {
  let mut closest = max;
  let mut result: Option<(Collision, &'a Shadable)> = None;
  for index in indices {
    let element = &primitives[*index];
    match element.intersect(ray, first_hit_only, min, closest) {
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
  first_hit_only: bool,
  parent_min: f32,
  parent_max: f32,
) -> Option<(Collision, &'a Shadable)> {
  let mut stack: Vec<(&BVHNode, /*min*/ f32, /*max*/ f32)> = Vec::new();
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
            match intersect_primitives(
              children,
              elements,
              ray,
              first_hit_only,
              min.max(node_min) as f32,
              nearest.min(max as f32),
            ) {
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

const NUM_BUCKETS: usize = 4;
const MAX_PRIMS_PER_NODE: usize = 20;

#[derive(Copy, Clone, Debug)]
struct BucketInfo {
  pub count: usize,
  pub left_inclusive_count: usize,
  pub right_exclusive_count: usize,
  pub bounds: BoundingBox,
  pub centroid_bounds: BoundingBox,
  pub left_inclusive_bounds: BoundingBox,
  pub right_exclusive_bounds: BoundingBox,
  pub left_inclusive_centroid_bounds: BoundingBox,
  pub right_exclusive_centroid_bounds: BoundingBox,
  pub split_cost: f32,
}

fn bucket_for_primitive(
  centroid_bounds: BoundingBox,
  bucket_count: usize,
  axis: usize,
  primitive: &BVHPrimitiveInfo,
) -> usize {
  return ((bucket_count as f32 * centroid_bounds.offset(primitive.centroid).axis(axis) as f32) as usize)
    .min(bucket_count - 1);
}

fn recursive_build(depth: usize, primitives: &mut Vec<BVHPrimitiveInfo>) -> BVHNode {
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
    return make_leaf(&primitives);
  }

  let mut centroid_bounds = BoundingBox::new();
  for primitive in primitives.iter() {
    centroid_bounds = centroid_bounds.merge_with_point(primitive.centroid);
  }

  let max_axis = centroid_bounds.max_axis();
  if centroid_bounds.max.data.extract(max_axis) == centroid_bounds.min.data.extract(max_axis) {
    return make_leaf(&primitives);
  }

  let mut buckets = [BucketInfo {
    count: 0,
    left_inclusive_count: 0,
    right_exclusive_count: 0,
    bounds: BoundingBox::new(),
    centroid_bounds: BoundingBox::new(),
    left_inclusive_bounds: BoundingBox::new(),
    right_exclusive_bounds: BoundingBox::new(),
    left_inclusive_centroid_bounds: BoundingBox::new(),
    right_exclusive_centroid_bounds: BoundingBox::new(),
    split_cost: 0.0,
  }; NUM_BUCKETS];

  let initial_count = primitives.len();

  // First pass, accrue the per bucket primitive information
  for primitive in primitives.iter() {
    let b = bucket_for_primitive(centroid_bounds, NUM_BUCKETS, max_axis, primitive);
    assert!(b < NUM_BUCKETS);
    buckets[b].count += 1;
    buckets[b].bounds = buckets[b].bounds.merge_with_bbox(primitive.bounds);
    buckets[b].centroid_bounds = buckets[b].centroid_bounds.merge_with_bbox(primitive.bounds);
    assert!(buckets[b].bounds.is_valid());
  }

  // Second pass, compute left properties
  {
    let mut cummulative_bounds = BoundingBox::new();
    let mut cummulative_centroid_bounds = BoundingBox::new();
    let mut cummulative_count = 0;
    for i in 0..NUM_BUCKETS {
      buckets[i].left_inclusive_count = cummulative_count + buckets[i].count;
      cummulative_count = buckets[i].left_inclusive_count;
      buckets[i].left_inclusive_bounds = cummulative_bounds.merge_with_bbox(buckets[i].bounds);
      cummulative_bounds = buckets[i].left_inclusive_bounds;
      buckets[i].left_inclusive_centroid_bounds =
        cummulative_centroid_bounds.merge_with_bbox(buckets[i].left_inclusive_centroid_bounds);
      cummulative_centroid_bounds = buckets[i].left_inclusive_centroid_bounds;
    }
  }

  // Third pass, compute right properties
  {
    let mut prior_bounds = BoundingBox::new();
    let mut prior_centroid_bounds = BoundingBox::new();
    let mut cummulative_count = 0;
    for j in 0..NUM_BUCKETS {
      let i = NUM_BUCKETS - j - 1;
      buckets[i].right_exclusive_bounds = prior_bounds;
      prior_bounds = prior_bounds.merge_with_bbox(buckets[i].bounds);
      buckets[i].right_exclusive_centroid_bounds = prior_centroid_bounds;
      prior_centroid_bounds = prior_centroid_bounds.merge_with_bbox(buckets[i].right_exclusive_centroid_bounds);

      buckets[i].right_exclusive_count = cummulative_count;
      cummulative_count = cummulative_count + buckets[i].count;
    }
  }

  // Fourth pass, compute the split costs
  {
    let leaf_surface = bounds.surface_area();
    for bucket in buckets.iter_mut() {
      assert!(bucket.left_inclusive_count + bucket.right_exclusive_count == initial_count);
      let left_cost = bucket.left_inclusive_bounds.surface_area() / leaf_surface * bucket.left_inclusive_count as f32;
      let right_cost =
        bucket.right_exclusive_bounds.surface_area() / leaf_surface * bucket.right_exclusive_count as f32;
      bucket.split_cost = 1.0 + left_cost * 2.0 + right_cost * 2.0;
    }
  }

  let mut minimum_split_cost = std::f32::INFINITY;
  let mut minimum_split_bucket = 0;
  for i in 0..NUM_BUCKETS - 1 {
    if buckets[i].split_cost < minimum_split_cost {
      minimum_split_bucket = i;
      minimum_split_cost = buckets[i].split_cost;
    }
  }

  let leaf_cost = initial_count as f32;
  if leaf_cost < minimum_split_cost && initial_count <= MAX_PRIMS_PER_NODE {
    return make_leaf(&primitives);
  }

  let mut left_primitives: Vec<BVHPrimitiveInfo> = primitives
    .drain_filter(|primitive| {
      let b = bucket_for_primitive(centroid_bounds, NUM_BUCKETS, max_axis, primitive);
      return b <= minimum_split_bucket;
    })
    .collect();
  let mut right_primitives = primitives;

  assert!(left_primitives.len() != 0);
  assert!(right_primitives.len() != 0);
  assert!(left_primitives.len() + right_primitives.len() == initial_count);
  let left_child = Box::new(recursive_build(depth + 1, &mut left_primitives));
  let right_child = Box::new(recursive_build(depth + 1, &mut right_primitives));
  return BVHNode::Node((bounds, max_axis, left_child, right_child));
}
