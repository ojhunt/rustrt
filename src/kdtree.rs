use vectors::Vec4d;
use bounding_box::BoundingBox;
use bounding_box::HasBoundingBox;
use heap::PriorityHeap;

pub trait HasPosition {
  fn get_position(&self) -> Vec4d;
}

#[derive(Debug)]
struct KDTreeInnerNode<T> {
  children: Box<[KDTreeNode<T>; 2]>,
  axis: usize,
  value: f64,
  bounds: BoundingBox,
}

#[derive(Debug)]
enum KDTreeNode<T> {
  Node(KDTreeInnerNode<T>),
  Leaf(Vec<T>, BoundingBox),
}

fn shortest_distance(position: Vec4d, bounds: BoundingBox) -> f64 {
  let min = bounds.min - position;
  let max = position - bounds.max;
  let nearest_vec = Vec4d::vector(
    min.x.max(0.).max(max.x),
    min.y.max(0.).max(max.y),
    min.z.max(0.).max(max.z),
  );
  return nearest_vec.length();
}

fn merge_nearest<T: Clone + HasPosition>(
  near: (&[T], BoundingBox),
  far: Option<(&[T], BoundingBox)>,
  position: Vec4d,
  count: usize,
) -> (Vec<T>, BoundingBox, f64) {
  let mut result_bounds = BoundingBox::new();
  let mut near_vec = near.0.to_vec();
  assert!(near_vec.len() > 0);
  let comparator = |l: &T, r: &T| {
    let left_distance = (l.get_position() - position).length();
    let right_distance = (r.get_position() - position).length();
    left_distance.partial_cmp(&right_distance).unwrap()
  };
  near_vec.sort_by(comparator);
  near_vec.truncate(count);
  let near_bounds = {
    let mut b = BoundingBox::new();
    for elem in &near_vec {
      b = b.merge_with_point(elem.get_position());
    }
    b
  };

  let current_worst = (near_vec.last().unwrap().get_position() - position).length();

  let (far_elements, far_bounds) = match far {
    None => return (near_vec, near_bounds, current_worst),
    Some(x) => x,
  };

  let far_nearest = shortest_distance(position, far_bounds);
  if near_vec.len() == count && far_nearest > current_worst {
    return (near_vec, near_bounds, current_worst);
  }
  near_vec.extend_from_slice(far_elements);
  near_vec.sort_by(comparator);
  near_vec.truncate(count);

  let current_worst = (near_vec.last().unwrap().get_position() - position).length();
  let near_bounds = {
    let mut b = BoundingBox::new();
    for elem in &near_vec {
      b = b.merge_with_point(elem.get_position());
    }
    b
  };
  return (near_vec, near_bounds, current_worst);
}

impl<T: Clone + HasPosition> KDTreeNode<T> {
  fn depth(&self) -> (usize, usize) {
    match self {
      KDTreeNode::Node(n) => {
        let (left_min, left_max) = n.children[0].depth();
        let (right_min, right_max) = n.children[1].depth();
        (1 + left_min.min(right_min), 1 + left_max.max(right_max))
      }
      KDTreeNode::Leaf(_, _) => (1, 1),
    }
  }

  fn bounds(&self) -> BoundingBox {
    match self {
      KDTreeNode::Node(n) => n.bounds,
      KDTreeNode::Leaf(_, b) => *b,
    }
  }

  // Far from optimal -- the furthest node should start its calculation on top of the existing list
  fn nearest(&self, position: Vec4d, count: usize, max_distance: f64) -> (Vec<T>, BoundingBox, f64) {
    let node = match self {
      KDTreeNode::Leaf(elements, bounds) => {
        return merge_nearest((elements, *bounds), None, position, count);
      }
      KDTreeNode::Node(node) => node,
    };
    let (nearest_child, farthest_child) = {
      if position[node.axis] < node.value {
        (&node.children[0], &node.children[1])
      } else {
        (&node.children[1], &node.children[0])
      }
    };

    let (nearest_elements, nearest_bounds, current_worst) = nearest_child.nearest(position, count, std::f64::INFINITY);
    if nearest_elements.len() == count && shortest_distance(position, farthest_child.bounds()) > current_worst {
      return (nearest_elements, nearest_bounds, current_worst);
    }

    let worst_for_far_branch = {
      if nearest_elements.len() == count {
        current_worst
      } else {
        std::f64::INFINITY
      }
    };
    let (farthest_elements, farthest_bounds, farthest_worse) =
      farthest_child.nearest(position, count, worst_for_far_branch);
    return merge_nearest(
      (&nearest_elements, nearest_bounds),
      Some((&farthest_elements, farthest_bounds)),
      position,
      count,
    );
  }
}

#[derive(Debug)]
pub struct KDTree<T: Clone + HasPosition> {
  root: KDTreeNode<T>,
}

impl<T: Clone + HasPosition> KDTree<T> {
  pub fn depth(&self) -> (usize, usize) {
    self.root.depth()
  }
}

fn build_tree<T: Clone + HasBoundingBox + HasPosition>(
  elements: &[T],
  bounds: BoundingBox,
  max_children: usize,
) -> KDTreeNode<T> {
  if elements.len() < max_children {
    return KDTreeNode::Leaf(elements.to_vec(), bounds);
  }
  let max_axis = bounds.max_axis();

  let mut copy = elements.to_vec();
  copy.sort_by(|l, r| {
    let left = l.get_position()[max_axis];
    let right = r.get_position()[max_axis];
    return left.partial_cmp(&right).unwrap();
  });
  let split_point = copy[copy.len() / 2].get_position()[max_axis];

  let (left, right) = copy.split_at(copy.len() / 2);

  let left_bounds = {
    let mut result = BoundingBox::new();
    for elem in left {
      result = result.merge_with_point(elem.get_position());
    }
    result
  };
  let right_bounds = {
    let mut result = BoundingBox::new();
    for elem in right {
      result = result.merge_with_point(elem.get_position());
    }
    result
  };
  return KDTreeNode::Node(KDTreeInnerNode {
    axis: max_axis,
    value: split_point,
    children: Box::new([
      build_tree(left, left_bounds, max_children),
      build_tree(right, right_bounds, max_children),
    ]),
    bounds: bounds,
  });
}

impl<T: Clone + HasBoundingBox + HasPosition> KDTree<T> {
  pub fn new(elements: &[T], max_children: usize) -> Self {
    let mut bounds = BoundingBox::new();
    for elem in elements {
      bounds = bounds.merge_with_bbox(elem.bounds());
    }
    println!("Max children: {}", max_children);
    return KDTree {
      root: build_tree(elements, bounds, max_children),
    };
  }
  pub fn nearest(&self, position: Vec4d, count: usize) -> (Vec<T>, f64) {
    let (elements, _, distance) = self.root.nearest(position, count, std::f64::INFINITY);
    return (elements, distance);
  }
}
