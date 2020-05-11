use crate::morton;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct BVHNode {
    pub center: [f32; 4],
    pub radius: f32,
    pub left_child: u32,
    pub right_child: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct BVHLeaf {
    pub inv_model: [[f32; 4]; 4],
    pub model_address: u32,
}

#[derive(Debug)]
pub struct BoundingVolumeHierarchy {
    pub root: u32,
    pub nodes: Vec<BVHNode>,
    pub leafs: Vec<BVHLeaf>,
}

unsafe impl bytemuck::Pod for BVHNode {}
unsafe impl bytemuck::Zeroable for BVHNode {}
unsafe impl bytemuck::Pod for BVHLeaf {}
unsafe impl bytemuck::Zeroable for BVHLeaf {}

impl BoundingVolumeHierarchy {
    pub fn new(objects: &[(u32, na::Similarity3<f32>)]) -> BoundingVolumeHierarchy {
        let mut morton_leafs = Vec::<(u64, BVHLeaf, na::Similarity3<f32>, f32)>::new();
        for (address, transform) in objects {
            let position = transform.isometry.translation.vector * 10.0;
            let scale = transform.scaling();
            let morton = morton::encode_3d(position.x as u64, position.y as u64, position.z as u64);
            let leaf = BVHLeaf {
                model_address: *address,
                inv_model: transform.to_homogeneous().try_inverse().unwrap().into(),
            };
            morton_leafs.push((morton, leaf, *transform, scale));
        }
        morton_leafs.sort_by_key(|x| x.0);

        let mut leafs = Vec::<BVHLeaf>::new();
        let mut nodes = Vec::<BVHNode>::new();
        let mut node_queue = std::collections::VecDeque::<usize>::new();
        nodes.push(BVHNode {
            center: [0.0; 4],
            radius: 0.0,
            left_child: 0,
            right_child: 0,
        });
        let empty_node_idx = 0;
        for i in (0..morton_leafs.len()).step_by(2) {
            leafs.push(morton_leafs[i].1);
            let (center, radius, left_child, right_child) = {
                if i == morton_leafs.len() - 1 {
                    let (center, radius) = center_and_radius(&morton_leafs[i].2);
                    (center, radius, i as u32, empty_node_idx)
                } else {
                    leafs.push(morton_leafs[i + 1].1);
                    let (center, radius) = bounding_sphere(&[
                        &center_and_radius(&morton_leafs[i].2),
                        &center_and_radius(&morton_leafs[i + 1].2),
                    ]);
                    (center, radius, i as u32, (i + 1) as u32)
                }
            };
            nodes.push(BVHNode {
                center: [center.x, center.y, center.z, center.w],
                radius,
                left_child: left_child | 0x80000000,
                right_child: right_child | 0x80000000,
            });
            node_queue.push_back(nodes.len() - 1);
        }

        while node_queue.len() > 1 {
            let left_child = node_queue.pop_front().unwrap();
            let right_child = node_queue.pop_front().unwrap();
            let left_node = &nodes[left_child];
            let right_node = &nodes[right_child];
            let (center, radius) = bounding_sphere(&[
                &(left_node.center.into(), left_node.radius),
                &(right_node.center.into(), right_node.radius),
            ]);
            nodes.push(BVHNode {
                center: [center.x, center.y, center.z, center.w],
                radius,
                left_child: left_child as u32,
                right_child: right_child as u32,
            });
            node_queue.push_back(nodes.len() - 1);
        }
        let root = node_queue.pop_front().unwrap() as u32;
        assert!(node_queue.is_empty());
        BoundingVolumeHierarchy { root, nodes, leafs }
    }
}

fn center_and_radius(transform: &na::Similarity3<f32>) -> (na::Point4<f32>, f32) {
    (
        transform
            .isometry
            .translation
            .vector
            .to_homogeneous()
            .into(),
        transform.scaling(),
    )
}

fn bounding_sphere(nodes: &[&(na::Point4<f32>, f32)]) -> (na::Point4<f32>, f32) {
    let inf = std::f32::INFINITY;
    let mut min = na::Point4::new(inf, inf, inf, 1.0);
    let mut max = na::Point4::new(-inf, -inf, -inf, 1.0);
    for i in 0..2 {
        let node_center = nodes[i].0;
        let node_radius = nodes[i].1;
        min.x = f32::min(min.x, node_center.x - node_radius);
        min.y = f32::min(min.y, node_center.y - node_radius);
        min.z = f32::min(min.z, node_center.z - node_radius);
        max.x = f32::max(max.x, node_center.x + node_radius);
        max.y = f32::max(max.y, node_center.y + node_radius);
        max.z = f32::max(max.z, node_center.z + node_radius);
    }
    let radius = na::distance(&min, &max) / 2.0;
    let center = na::center(&min, &max);
    (center, radius)
}
