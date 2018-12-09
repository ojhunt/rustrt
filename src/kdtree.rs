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

fn build_tree<T: Clone>(elements: &[T]) -> KDTreeNode<T> {
    return KDTreeNode::Leaf(elements.to_vec());
}

impl<T: Clone> KDTree<T> {
    pub fn new(elements: &[T]) -> Self {
        return KDTree {
            root: build_tree(elements),
        };
    }
}
