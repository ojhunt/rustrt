use vectors::Vec4d;
use bounding_box::BoundingBox;
use bounding_box::HasBoundingBox;

pub trait HasPosition {
  fn get_position(&self) -> Vec4d;
}

#[derive(Debug)]
struct KDTreeInnerNode<T> {
  children: Box<[KDTreeNode<T>; 2]>,
  axis: usize,
  value: f64,
}

#[derive(Debug)]
enum KDTreeNode<T> {
  Node(KDTreeInnerNode<T>),
  Leaf(Vec<T>),
}

impl<T> KDTreeNode<T> {
  fn depth(&self) -> (usize, usize) {
    match self {
      KDTreeNode::Node(n) => {
        let (left_min, left_max) = n.children[0].depth();
        let (right_min, right_max) = n.children[1].depth();
        (1 + left_min.min(right_min), 1 + left_max.max(right_max))
      }
      KDTreeNode::Leaf(_) => (1, 1),
    }
  }
}

#[derive(Debug)]
pub struct KDTree<T: Clone> {
  root: KDTreeNode<T>,
}

impl<T: Clone> KDTree<T> {
  pub fn depth(&self) -> (usize, usize) {
    self.root.depth()
  }
}

fn build_tree<T: Clone + HasBoundingBox + HasPosition>(elements: &[T], bounds: BoundingBox) -> KDTreeNode<T> {
  if elements.len() < 8 {
    return KDTreeNode::Leaf(elements.to_vec());
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
    children: Box::new([build_tree(left, left_bounds), build_tree(right, right_bounds)]),
  });
}

impl<T: Clone + HasBoundingBox + HasPosition> KDTree<T> {
  pub fn new(elements: &[T]) -> Self {
    let mut bounds = BoundingBox::new();
    for elem in elements {
      bounds = bounds.merge_with_bbox(elem.bounds());
    }
    return KDTree {
      root: build_tree(elements, bounds),
    };
  }
}
