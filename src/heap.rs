use std::cmp::Ordering;

pub type Comparator<T> = Fn(&T, &T) -> Ordering;

pub struct PriorityHeap<'a, T: Clone> {
  max_size: usize,
  data: Vec<T>,
  comparator: &'a Comparator<T>,
}

impl<'a, T: Clone> PriorityHeap<'a, T> {
  pub fn new(comparator: &'a Comparator<T>, max_size: usize) -> PriorityHeap<'a, T> {
    PriorityHeap {
      max_size,
      data: vec![],
      comparator,
    }
  }
  pub fn append_buffer(&mut self, buffer: &mut Vec<T>) {
    buffer.sort_by(|a, b| (self.comparator)(a, b).reverse());
    self.data.append(buffer);
  }
  pub fn top(&self) -> Option<&T> {
    if self.data.len() == 0 {
      None
    } else {
      Some(&self.data[0])
    }
  }
  pub fn pop(&mut self) -> Option<T> {
    if self.data.len() == 0 {
      return None;
    }
    let result = Some(self.data[0].clone());
    if self.data.len() == 1 {
      self.data = vec![];
      return result;
    }

    let temp = self.data.last().unwrap().clone();
    let data = &mut self.data;
    {
      data[0] = temp.clone();
    }

    let length = data.len() - 1;
    data.truncate(length);

    let mut current_index = 0;

    while current_index * 2 < data.len() - 1 {
      let left_child_index = current_index * 2 + 1;
      let left = data[left_child_index].clone();
      let right_child_index = current_index * 2 + 2;
      if right_child_index == data.len() {
        if (self.comparator)(&temp, &left) != Ordering::Greater {
          data.swap(current_index, left_child_index);
        }
        return result;
      }
      let right = data[right_child_index].clone();
      if (self.comparator)(&temp, &left) == Ordering::Greater && (self.comparator)(&temp, &right) == Ordering::Greater {
        return result;
      }
      let greater_index = {
        if (self.comparator)(&left, &right) == Ordering::Greater {
          left_child_index
        } else {
          right_child_index
        }
      };
      data.swap(current_index, greater_index);

      current_index = greater_index;
    }
    return result;
  }

  fn internal_insert(&mut self, new: &T) {
    assert!(self.data.len() < self.max_size);
    let data = &mut self.data;
    let mut current_index = data.len();
    data.push(new.clone());
    while current_index > 0 {
      let parent_index = (current_index - 1) / 2;
      let parent = data[parent_index].clone();
      if (self.comparator)(new, &parent) == Ordering::Less {
        return;
      }
      data.swap(current_index, parent_index);
      current_index = parent_index;
    }
  }

  pub fn insert(&mut self, new: T) {
    if self.data.len() >= self.max_size {
      if (self.comparator)(&self.data[0], &new) == Ordering::Less {
        return;
      }
      self.pop();
    }
    self.internal_insert(&new);
  }

  pub fn slice(&self) -> &[T] {
    return &self.data;
  }
}

#[test]
fn test_sort() {
  let comparator = |a: &usize, b: &usize| a.cmp(b);
  let mut heap: PriorityHeap<usize> = PriorityHeap::new(&comparator, 10);
  for i in 0..10 {
    heap.insert((i * 17 + 31) % 43);
  }
  let mut result: Vec<usize> = vec![];
  while let Some(top) = heap.pop() {
    result.push(top);
  }
  for i in 1..result.len() {
    assert!(result[i - 1] >= result[i]);
  }
}

#[test]
fn test_sort1() {
  let comparator = |a: &usize, b: &usize| a.cmp(b);
  let mut heap: PriorityHeap<usize> = PriorityHeap::new(&comparator, 10);
  for i in 0..10 {
    heap.insert(i);
  }
  let mut result: Vec<usize> = vec![];
  while let Some(top) = heap.pop() {
    result.push(top);
  }
  for i in 1..result.len() {
    assert!(result[i - 1] >= result[i]);
  }
  for i in 0..10 {
    assert_eq!(result[10 - i - 1], i);
  }
}
#[test]
fn test_sort2() {
  let comparator = |a: &usize, b: &usize| a.cmp(b);
  let mut heap: PriorityHeap<usize> = PriorityHeap::new(&comparator, 5);
  for i in 0..10 {
    heap.insert(i);
  }
  let mut result: Vec<usize> = vec![];
  while let Some(top) = heap.pop() {
    result.push(top);
  }
  for i in 1..result.len() {
    assert!(result[i - 1] >= result[i]);
  }
  assert_eq!(result, [4, 3, 2, 1, 0]);
}

#[test]
fn test_sort3() {
  let comparator = |a: &usize, b: &usize| a.cmp(b);
  let mut heap: PriorityHeap<usize> = PriorityHeap::new(&comparator, 5);
  for i in 0..10 {
    heap.insert(10 - 1 - i);
    heap.insert(10 - 1 - i);
  }
  let mut result: Vec<usize> = vec![];
  while let Some(top) = heap.pop() {
    result.push(top);
  }
  for i in 1..result.len() {
    assert!(result[i - 1] >= result[i]);
  }
  assert_eq!(result, [2, 1, 1, 0, 0]);
}
