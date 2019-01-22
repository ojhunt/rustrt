use std::cmp::Ordering;
use heap::Comparator;
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
  value: f32,
  bounds: BoundingBox,
}

#[derive(Debug)]
enum KDTreeNode<T> {
  Node(KDTreeInnerNode<T>),
  Leaf(Vec<T>, BoundingBox),
}

struct ElementAccumulator<'a, T: Clone> {
  heap: PriorityHeap<'a, T>,
  data: Vec<T>,
  max_count: usize,
  count: usize,
  top: Option<usize>,
  comparator: &'a Comparator<T>,
}
impl<'a, T: Clone> ElementAccumulator<'a, T> {
  pub fn new(c: &'a Comparator<T>, max_count: usize) -> Self {
    Self {
      heap: PriorityHeap::new(c, max_count),
      data: vec![],
      max_count,
      count: 0,
      comparator: c,
      top: None,
    }
  }
  pub fn is_empty(&self) -> bool {
    return self.count == 0;
  }
  pub fn slice(&self) -> &[T] {
    if self.count == self.max_count {
      return self.heap.slice();
    }
    return &self.data;
  }
  pub fn top(&self) -> Option<&T> {
    if self.count < self.max_count {
      return match self.top {
        Some(index) => Some(&self.data[index]),
        None => None,
      };
    }
    return self.heap.top();
  }
  pub fn is_full(&self) -> bool {
    return self.count == self.max_count;
  }
  pub fn insert(&mut self, new_value: T) {
    if self.count == self.max_count {
      self.heap.insert(new_value);
      return;
    }

    if let Some(other) = self.top {
      if (self.comparator)(&new_value, &self.data[other]) == Ordering::Less {
        self.top = Some(self.data.len());
      }
    } else {
      self.top = Some(self.data.len());
    }
    assert!(self.top.is_some());
    self.data.push(new_value);
    self.count = self.data.len();

    if self.count == self.max_count {
      self.heap.append_buffer(&mut self.data);
    }
  }
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

  // Far from optimal -- the furthest node should start its calculation on top of the existing list
  fn nearest(&self, nearest_elements: &mut ElementAccumulator<(f64, T)>, position: Vec4d, max_distance: f64) {
    let node = match self {
      KDTreeNode::Leaf(elements, bounds) => {
        if nearest_elements.is_full() {
          let distance = Vec4d::splat(nearest_elements.top().unwrap().0 as f32);
          let min = bounds.min - distance;
          let max = bounds.max + distance;
          if position.lt(min).any() || position.gt(max).any() {
            return;
          }
        }

        for element in elements {
          let distance = (position - element.get_position()).length();
          if distance <= max_distance {
            nearest_elements.insert((distance, element.clone()));
          }
        }
        return;
      }
      KDTreeNode::Node(node) => node,
    };

    let (nearest_child, farthest_child, left_of_split) = {
      if position.data.extract(node.axis) < node.value {
        (&node.children[0], &node.children[1], true)
      } else {
        (&node.children[1], &node.children[0], false)
      }
    };
    nearest_child.nearest(nearest_elements, position, max_distance);
    if nearest_elements.is_full() {
      if let Some((distance, _)) = nearest_elements.top() {
        if left_of_split {
          if position.data.extract(node.axis) + (*distance as f32) < node.value {
            return;
          }
        } else {
          if position.data.extract(node.axis) - (*distance as f32) > node.value {
            return;
          }
        }
      }
    }
    farthest_child.nearest(nearest_elements, position, max_distance);
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
    let left = l.get_position().data.extract(max_axis);
    let right = r.get_position().data.extract(max_axis);
    return left.partial_cmp(&right).unwrap();
  });
  let split_point = copy[copy.len() / 2].get_position().data.extract(max_axis);

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
  pub fn nearest(&self, position: Vec4d, count: usize, max_distance: f64) -> (Vec<(T, f64)>, f64) {
    let comparator = |a: &(f64, T), b: &(f64, T)| return a.0.partial_cmp(&b.0).unwrap();
    let mut queue: ElementAccumulator<(f64, T)> = ElementAccumulator::new(&comparator, count);
    self.root.nearest(&mut queue, position, max_distance);
    if queue.is_empty() {
      return (vec![], std::f64::INFINITY);
    }
    let result = queue.slice().iter().map(|(dist, elem)| (elem.clone(), *dist)).collect();
    return (result, queue.top().unwrap().0);
  }
}
