use bounding_box::BoundingBox;
use bounding_box::HasBoundingBox;

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

#[derive(Debug)]
pub struct KDTree<T: Clone> {
  root: KDTreeNode<T>,
}

fn build_tree<T: Clone + HasBoundingBox>(elements: &[T], bounds: BoundingBox) -> KDTreeNode<T> {
  // if elements.len() < 10 {
  return KDTreeNode::Leaf(elements.to_vec());
  // }
  // let mut longest_axis = bounds.max_axis();
  // let counts = [0; 16];
  // let base = bounds.min()[longest_axis];
  // let size = bounds.max()[longest_axis] - base;
  // panic!();
}

impl<T: Clone + HasBoundingBox> KDTree<T> {
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
