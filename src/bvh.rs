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

impl BVH {
   pub fn new<T: Intersectable>(elements: &[T]) -> BVH {
       let mut info : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
       let mut total_bounds = BoundingBox::new();
       for i in 0..elements.len() {
           let element = &elements[i];
           let inner_bounds = element.bounds();
           let centroid = inner_bounds.centroid();
           total_bounds = total_bounds.merge_with_bbox(inner_bounds);
           info.push((inner_bounds, centroid, i));
       }
       BVH{
           root: recursive_build(&info, total_bounds)
       }
   }
}


const max_elements_per_node: usize = 8;
fn recursive_build(
            elements: &[(BoundingBox, Vec4d, usize)], bounds: BoundingBox) -> BVHNode {

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

    const number_of_bins : usize = 16;
    let scale_constant = (number_of_bins as f64 * (1. - 0.0000001)) / (max_axis - axis_start);
    let mut bins : [(usize, BoundingBox); number_of_bins] = [(0, BoundingBox::new()); number_of_bins];
    elements.iter().map(
        |(bound, centroid, _)| {
            let offset = get_axis(*centroid) - axis_start;
            let index = (scale_constant * offset).max(0.0).floor() as usize;
            let (index_count, index_bounds) = bins[index];
            bins[index] = (index_count + 1, index_bounds.merge_with_bbox(*bound))
        }
    );

    let mut leftNodeCost = [0.0 as f64; number_of_bins];
    {
        let mut last_count = 0;
        let mut last_bounds = BoundingBox::new();
        for i in 0..number_of_bins {
            let (bin_count, bin_bounds) = bins[i];
            if bin_count == 0 {
                continue;
            }
            leftNodeCost[i] = last_count as f64 * last_bounds.surface_area();
            last_count += bin_count;
            last_bounds = last_bounds.merge_with_bbox(bin_bounds);
        }
    }
    let mut rightNodeCost = [0.0 as f64; number_of_bins];
    {
        let mut last_count = 0;
        let mut last_bounds = BoundingBox::new();
        for i in number_of_bins-1..=0 {
            let (bin_count, bin_bounds) = bins[i];
            if bin_count == 0 {
                continue;
            }
            last_count += bin_count;
            last_bounds = last_bounds.merge_with_bbox(bin_bounds);
            rightNodeCost[i] = last_count as f64 * last_bounds.surface_area();
        }
    }

    /*

     index | 0 | 1 | 2 | 3 | 4 | 5 |
     count | 1 | 1 | 0 | 2 | 1 | 1 |
     left  | 0 | 1 | 2 | 2 | 4 | 5 |
     right | 6 | 5 | 4 | 4 | 2 | 1 |

    */

    let mut sahCost = [0.0 as f64; number_of_bins];
    let node_sa = bounds.surface_area();
    for i in 0..number_of_bins {
        let left_cost = leftNodeCost[i];
        let right_cost = rightNodeCost[i];
        // this is the pbrt calculation, but the rest of the
        // code is weirdly inefficient - maybe just for clarity?
        sahCost[i] = 0.125 + (left_cost + right_cost) / node_sa;
    }

    let mut minimum_cost = sahCost[0];
    let mut split_point = 0;
    for i in 1..number_of_bins {
        if sahCost[i] < minimum_cost {
            minimum_cost = sahCost[i];
            split_point = i;
        }
    }
    let split_value = axis_start + (split_point as f64 * ( (max_axis - axis_start) / number_of_bins as f64));
    if elements.len() > max_elements_per_node || minimum_cost < elements.len() as f64 {
        let mut left_elements : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
        let mut right_elements : Vec<(BoundingBox, Vec4d, usize)> = Vec::new();
        let mut left_bounds = BoundingBox::new();
        let mut right_bounds = BoundingBox::new();
        for (bounds, centre, elem) in elements {
            if get_axis(*centre) < split_value {
                left_elements.push((*bounds, *centre, *elem));
                left_bounds = left_bounds.merge_with_bbox(*bounds);
            } else {
                right_elements.push((*bounds, *centre, *elem));
                right_bounds = right_bounds.merge_with_bbox(*bounds);
            }
        }
        let left_node = recursive_build(&left_elements, left_bounds);
        let right_node = recursive_build(&right_elements, right_bounds);
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