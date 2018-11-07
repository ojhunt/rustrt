use bounding_box::BoundingBox;
use vec4d::Vec4d;
use intersectable::Intersectable;

#[derive(Debug)]
enum BVHNode {
    Leaf((BoundingBox, Vec<usize>)),
    Node((BoundingBox, Box<BVHNode>, Box<BVHNode>))
}

#[derive(Debug)]
pub struct BVH {
    root: BVHNode
}

fn tree_depth(node: &BVHNode) -> usize {
    match node {
        BVHNode::Leaf(_) => 1,
        BVHNode::Node((_, left, right)) => 1 + tree_depth(left).max(tree_depth(right))
    }
}

impl BVH {
   pub fn new<T: Intersectable>(elements: &[T]) -> BVH {
       let mut info : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
       let mut total_bounds = BoundingBox::new();
       for i in 0..elements.len() {
           let element = &elements[i];
           let inner_bounds = element.bounds();
           let centroid = inner_bounds.centroid();
           total_bounds = total_bounds.merge_with_point(centroid);
           info.push((inner_bounds, centroid, i));
       }
       let root = recursive_build(0, &info, total_bounds);
       let max_depth = tree_depth(&root);
       println!("Done building, max depth: {}", max_depth);
       BVH {
           root: root
       }
   }
}

const max_elements_per_node: usize = 4;
fn recursive_build(
            depth: usize,
            elements: &[(BoundingBox, Vec4d, usize)], bounds: BoundingBox) -> BVHNode {
    print!("Depth: {}\n", depth);
    let get_x : fn (Vec4d) -> f64 = |centroid| centroid.x;
    let get_y : fn (Vec4d) -> f64 = |centroid| centroid.y;
    let get_z : fn (Vec4d) -> f64 = |centroid| centroid.z;
    
    let mut get_axis = &get_x;
    let mut max_axis = bounds.max.x - bounds.min.x;
    let mut axis_start = bounds.min.x;
    let y_size = bounds.max.y - bounds.min.y;
    let z_size = bounds.max.z - bounds.min.z;
    if y_size > max_axis {
        max_axis = y_size;
        get_axis = &get_y;
        axis_start = bounds.min.y;
    }
    if z_size > max_axis {
        max_axis = z_size;
        get_axis = &get_z;
        axis_start = bounds.min.z;
    }

    const NUMBER_OF_BINS : usize = 16;
    let scale_constant = (NUMBER_OF_BINS as f64 * (1. - 0.0000001)) / (max_axis - axis_start);
    let mut bins : [(usize, BoundingBox); NUMBER_OF_BINS] = [(0, BoundingBox::new()); NUMBER_OF_BINS];
    for (bound, centroid, _) in elements {
        let offset = get_axis(*centroid) - axis_start;
        assert!(offset >= 0.);
        assert!(offset <= max_axis);
        let index = ((scale_constant * offset).max(0.0).floor() as usize).min(NUMBER_OF_BINS - 1);

        let (index_count, index_bounds) = bins[index];
        bins[index] = (index_count + 1, index_bounds.merge_with_bbox(*bound))
    }

    let mut left_node_cost = [0.0 as f64; NUMBER_OF_BINS];
    {
        let mut last_count = 0;
        let mut last_bounds = BoundingBox::new();
        for i in 0..NUMBER_OF_BINS {
            let (bin_count, bin_bounds) = bins[i];
            if bin_count == 0 {
                continue;
            }
            left_node_cost[i] = last_count as f64 * last_bounds.surface_area();
            last_count += bin_count;
            last_bounds = last_bounds.merge_with_bbox(bin_bounds);
        }
        assert!(last_count == elements.len());
    }

    let mut right_node_cost = [0.0 as f64; NUMBER_OF_BINS];
    {
        let mut last_count = 0;
        let mut last_bounds = BoundingBox::new();
        println!("{:?}", bins);
        for i in (0..NUMBER_OF_BINS).rev() {
            println!("HERE with i = {}", i);
            let (bin_count, bin_bounds) = bins[i];
            if bin_count == 0 {
                continue;
            }
            last_count += bin_count;
            last_bounds = last_bounds.merge_with_bbox(bin_bounds);
            println!("lastbounds: {:?} SAH: {:?}", last_bounds, last_bounds.surface_area());
            right_node_cost[i] = last_count as f64 * last_bounds.surface_area();
            assert!( right_node_cost[i] != 0.0);
        }
        assert!(last_count == elements.len());
    }

    /*

     index | 0 | 1 | 2 | 3 | 4 | 5 |
     count | 1 | 1 | 0 | 2 | 1 | 1 |
     left  | 0 | 1 | 2 | 2 | 4 | 5 |
     right | 6 | 5 | 4 | 4 | 2 | 1 |

    */

    let mut sah_cost = [0.0 as f64; NUMBER_OF_BINS];
    let node_sa = bounds.surface_area();
    for i in 0..NUMBER_OF_BINS {
        let left_cost = left_node_cost[i];
        let right_cost = right_node_cost[i];
        println!("left cost: {}, right cost: {}", left_cost, right_cost);
        // this is the pbrt calculation, but the rest of the
        // code is weirdly inefficient - maybe just for clarity?
        sah_cost[i] = 0.125 + (left_cost + right_cost) / node_sa;
    }

    let mut split_point = 0;
    for i in 0..NUMBER_OF_BINS {
        if bins[i].0 != 0 {
            split_point = i;
        }
    }
    let mut minimum_cost = sah_cost[split_point];

    for i in split_point..NUMBER_OF_BINS {
        if bins[i].0 == 0 {
            continue;
        }
        if sah_cost[i] < minimum_cost {
            minimum_cost = sah_cost[i];
            split_point = i;
            println!("left count: {}, right count: {}", left_node_cost[i], right_node_cost[i]);
        }
    }
    let split_value = axis_start + (split_point as f64 * ( (max_axis - axis_start) / NUMBER_OF_BINS as f64));

    println!("Split point: {}, split_value: {}, minimum_cost: {}, count: {}", split_point, split_value, minimum_cost, elements.len());
    if elements.len() > max_elements_per_node || minimum_cost < elements.len() as f64 {
        
        let mut left_elements : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
        let mut right_elements : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
        let mut left_bounds = BoundingBox::new();
        let mut right_bounds = BoundingBox::new();
        for (bounds, centre, elem) in elements {
            if get_axis(*centre) < split_value {
                left_elements.push((*bounds, *centre, *elem));
                left_bounds = left_bounds.merge_with_point(*centre);
            } else {
                right_elements.push((*bounds, *centre, *elem));
                right_bounds = right_bounds.merge_with_point(*centre);
            }
        }
        println!("minimum_cost: {:?}", minimum_cost);
        println!("split: {:?}", split_value);
        println!("left: {:?}", left_elements.len());
        println!("right {:?}", right_elements.len());

        assert!(left_elements.len() + right_elements.len() == elements.len());
        assert!(left_elements.len() != 0);
        assert!(right_elements.len() != 0);
        let left_node = recursive_build(depth + 1, &left_elements, left_bounds);
        let right_node = recursive_build(depth + 1, &right_elements, right_bounds);
        return BVHNode::Node((left_bounds.merge_with_bbox(right_bounds), Box::new(left_node), Box::new(right_node)));
    } else {
        let mut final_bounds = BoundingBox::new();
        let mut leaves : Vec<usize> = Vec::new();
        for (bounds, _, element) in elements {
            final_bounds = final_bounds.merge_with_bbox(*bounds);
            leaves.push(*element);
        }
        return BVHNode::Leaf((final_bounds, leaves));
    }
}