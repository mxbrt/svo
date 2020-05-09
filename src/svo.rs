use nalgebra::{base::Vector3, geometry::Point3};

use crate::raycast::{RaycastHit, Raycastable};
use crate::voxel_grid::VoxelGrid;

pub type Node = [u32; 2];

fn create_node() -> Node {
    [
        1 << 31, // default is an empty node
        0,
    ]
}

fn set_node(node: &mut Node, is_empty: bool, is_leaf: bool, children: usize, color: u32) {
    node[0] = ((is_empty as u32) << 31) | ((is_leaf as u32) << 30) | (children as u32 & 0x3fffffff);
    node[1] = color;
}

fn is_empty(node: &Node) -> bool {
    (node[0] & 0x80000000) != 0
}

fn is_leaf(node: &Node) -> bool {
    (node[0] & 0x40000000) != 0
}

fn child_idx(node: &Node) -> usize {
    (node[0] & 0x3fffffff) as usize
}

#[derive(Clone)]
pub struct SparseVoxelOctree {
    pub node_pool: Vec<Node>,
    pub size: Vector3<f32>,
}

impl SparseVoxelOctree {
    fn build_octree(
        &mut self,
        voxel_grid: &VoxelGrid,
        x: usize,
        y: usize,
        z: usize,
        size: usize,
    ) -> usize {
        let half_size = size >> 1;

        #[rustfmt::skip]
        let (cx,cy, cz) = // Offsets of child cubes
            ([x + half_size, x, x + half_size, x, x + half_size, x, x + half_size, x],
             [y + half_size, y + half_size, y, y, y + half_size, y + half_size, y, y],
             [z + half_size, z + half_size, z + half_size, z + half_size, z, z, z, z]);

        let node_tile_idx = self.node_pool.len();
        self.node_pool.append(&mut vec![create_node(); 8]);

        for i in 0..8 {
            if voxel_grid.sample(cx[i], cy[i], cz[i], half_size) {
                let color = (cx[i] | (cy[i] << 8) | (cz[i] << 16)) as u32;
                //let color = 0xFFFFFF;
                if half_size != 1 {
                    let child_idx = self.build_octree(voxel_grid, cx[i], cy[i], cz[i], half_size);
                    set_node(
                        &mut self.node_pool[node_tile_idx + i],
                        false,
                        false,
                        child_idx,
                        color,
                    );
                } else {
                    set_node(
                        &mut self.node_pool[node_tile_idx + i],
                        false,
                        true,
                        0x3fffffff,
                        color,
                    );
                }
            }
        }
        return node_tile_idx;
    }

    fn raymarch(
        &self,
        mut o: Point3<f32>,
        mut d: Vector3<f32>,
        t: &mut f32,
        color: &mut u32,
        normal: &mut Vector3<f32>,
    ) -> bool {
        // Maximum scale (number of float mantissa bits).
        const S_MAX: u32 = 23;
        const EPSILON: f32 = 1e-4; // TODO use exp2f(-S_MAX)
        let mut stack: [(usize, f32); (S_MAX + 1) as usize] = [(0, 0.0); (S_MAX + 1) as usize];

        o.x += 1.0;
        o.y += 1.0;
        o.z += 1.0;

        // Get rid of small ray direction components to avoid division by zero.
        // TODO copysignf
        if d.x.abs() < EPSILON {
            d.x = EPSILON
        }
        if d.y.abs() < EPSILON {
            d.y = EPSILON
        }
        if d.z.abs() < EPSILON {
            d.z = EPSILON
        }

        // Precompute the coefficients of tx(x), ty(y), and tz(z).
        let tx_coef: f32 = 1.0 / -d.x.abs();
        let ty_coef: f32 = 1.0 / -d.y.abs();
        let tz_coef: f32 = 1.0 / -d.z.abs();

        let mut tx_bias: f32 = tx_coef * o.x;
        let mut ty_bias: f32 = ty_coef * o.y;
        let mut tz_bias: f32 = tz_coef * o.z;

        let mut octant_mask: u32 = 7;
        if d.x > 0.0 {
            octant_mask ^= 1;
            tx_bias = 3.0 * tx_coef - tx_bias;
        }
        if d.y > 0.0 {
            octant_mask ^= 2;
            ty_bias = 3.0 * ty_coef - ty_bias;
        }
        if d.z > 0.0 {
            octant_mask ^= 4;
            tz_bias = 3.0 * tz_coef - tz_bias;
        }

        // Initialize the active span of t-values.
        let mut t_min: f32 = (2.0 * tx_coef - tx_bias)
            .max(2.0 * ty_coef - ty_bias)
            .max(2.0 * tz_coef - tz_bias);
        let mut t_max: f32 = (tx_coef - tx_bias)
            .min(ty_coef - ty_bias)
            .min(tz_coef - tz_bias);
        t_min = t_min.max(0.0);

        let mut parent_idx = 0;
        let mut cur = 0;
        let mut cur_node = self.node_pool[0];
        let mut idx: u32 = 0;
        let mut pos = Point3::<f32>::new(1.0, 1.0, 1.0);
        let mut scale: u32 = S_MAX - 1;
        let mut scale_exp2: f32 = 0.5; // exp2f(scale - s_max)
        let mut step_mask: u32 = 0;

        if 1.5 * tx_coef - tx_bias > t_min {
            idx ^= 1;
            pos.x = 1.5;
        }
        if 1.5 * ty_coef - ty_bias > t_min {
            idx ^= 2;
            pos.y = 1.5;
        }
        if 1.5 * tz_coef - tz_bias > t_min {
            idx ^= 4;
            pos.z = 1.5;
        }

        // Traverse voxels along the ray as long as the current voxel stays within the octree
        while scale < S_MAX {
            // Fetch child descriptor unless it is already valid
            if cur == 0 {
                cur_node = self.node_pool[parent_idx];
            }

            // Determine maximum t-value of the cube by evaluating
            // tx(), ty(), and tz() at its corner.
            let tx_corner: f32 = pos.x * tx_coef - tx_bias;
            let ty_corner: f32 = pos.y * ty_coef - ty_bias;
            let tz_corner: f32 = pos.z * tz_coef - tz_bias;
            let tc_max = tx_corner.min(ty_corner).min(tz_corner);

            // Process voxel if it exists and the active t-span is non-empty.
            let child_idx = child_idx(&cur_node) + (idx ^ octant_mask) as usize;
            let child = self.node_pool[child_idx];

            if !is_empty(&child) && t_min <= t_max {
                //// TODO Terminate if the voxel is small enough.
                //if tc_max * ray_size_coef + ray_size_bias >= scale_exp2 {
                //break;
                //}

                // INTERSECT
                // Intersect active t-span with the cube and evaluate
                // tx(), ty(), and tz() at the center of the voxel.
                let tv_max: f32 = t_max.min(tc_max);
                let half: f32 = scale_exp2 * 0.5;
                let tx_center: f32 = half * tx_coef + tx_corner;
                let ty_center: f32 = half * ty_coef + ty_corner;
                let tz_center: f32 = half * tz_coef + tz_corner;

                // Descend to the first child if the resulting t-span is non-empty.
                if t_min <= tv_max {
                    if is_leaf(&child) {
                        cur_node = child;
                        break;
                    }

                    // PUSH
                    // Write current parent to the stack
                    stack[scale as usize] = (parent_idx, t_max);
                    parent_idx = child_idx;
                    idx = 0;
                    scale -= 1;
                    scale_exp2 = half;

                    if tx_center > t_min {
                        idx ^= 1;
                        pos.x += scale_exp2;
                    }
                    if ty_center > t_min {
                        idx ^= 2;
                        pos.y += scale_exp2;
                    }
                    if tz_center > t_min {
                        idx ^= 4;
                        pos.z += scale_exp2;
                    }
                    // Update active t-span and invalidate cached child descriptor.
                    t_max = tv_max;
                    cur = 0;
                    continue;
                }
            }
            // ADVANCE
            // Step along the ray
            step_mask = 0;
            if tx_corner <= tc_max {
                step_mask ^= 1;
                pos.x -= scale_exp2;
            }
            if ty_corner <= tc_max {
                step_mask ^= 2;
                pos.y -= scale_exp2;
            }
            if tz_corner <= tc_max {
                step_mask ^= 4;
                pos.z -= scale_exp2;
            }

            // Update active t-span and flip bits of the child slot index.
            t_min = tc_max;
            idx ^= step_mask;

            // Proceed with pop if the bit flips disagree with the ray direction.
            if (idx & step_mask) != 0 {
                // POP
                // Find the highest differing bit between the two positions.
                let mut differing_bits: u32 = 0;
                if (step_mask & 1) != 0 {
                    differing_bits |= pos.x.to_bits() ^ (pos.x + scale_exp2).to_bits();
                }
                if (step_mask & 2) != 0 {
                    differing_bits |= pos.y.to_bits() ^ (pos.y + scale_exp2).to_bits();
                }
                if (step_mask & 4) != 0 {
                    differing_bits |= pos.z.to_bits() ^ (pos.z + scale_exp2).to_bits();
                }
                scale = 31 - (differing_bits).leading_zeros(); // position of the highest bit
                scale_exp2 = f32::from_bits(((scale as i32 - S_MAX as i32 + 127) << 23) as u32); // exp2f(scale - s_max)

                // Restore parent voxel from the stack.
                parent_idx = stack[scale as usize].0;
                t_max = stack[scale as usize].1;

                // Round cube position and extract child slot index.
                let shx: u32 = pos.x.to_bits() >> scale;
                let shy: u32 = pos.y.to_bits() >> scale;
                let shz: u32 = pos.z.to_bits() >> scale;
                pos.x = f32::from_bits(shx << scale);
                pos.y = f32::from_bits(shy << scale);
                pos.z = f32::from_bits(shz << scale);
                idx = (shx & 1) | ((shy & 1) << 1) | ((shz & 1) << 2);
                cur = 0;
            }
        }

        if scale >= S_MAX {
            return false;
        }

        // this happens for boundary voxels
        if step_mask == 0 {
            if 2.0 * tx_coef - tx_bias >= t_min {
                step_mask ^= 1;
            }
            if 2.0 * ty_coef - ty_bias >= t_min {
                step_mask ^= 2;
            }
            if 2.0 * tz_coef - tz_bias >= t_min {
                step_mask ^= 4;
            }
        }

        let face;
        if (octant_mask & 1) == 0 && (step_mask & 1) != 0 {
            face = 3;
        } else if (octant_mask & 2) == 0 && (step_mask & 2) != 0 {
            face = 5;
        } else if (octant_mask & 4) == 0 && (step_mask & 4) != 0 {
            face = 6;
        } else {
            face = step_mask;
        }

        match face {
            // right
            1 => normal.x = 1.0,
            // top
            2 => normal.y = 1.0,
            // left
            3 => normal.x = -1.0,
            // back
            4 => normal.z = 1.0,
            // bottom
            5 => normal.y = -1.0,
            // front
            6 => normal.z = -1.0,
            _ => (),
        }

        *t = t_min;
        *color = cur_node[1];
        return true;
    }
}

impl Raycastable for SparseVoxelOctree {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>) -> Option<RaycastHit> {
        let mut color = 0;
        let mut normal = Vector3::zeros();
        let mut t = 0.0;
        if self.raymarch(origin, dir, &mut t, &mut color, &mut normal) {
            return Some(RaycastHit {
                color,
                normal,
                pos: origin + t * dir,
            });
        } else {
            return None;
        }
    }
}

impl From<&VoxelGrid> for SparseVoxelOctree {
    fn from(voxel_grid: &VoxelGrid) -> SparseVoxelOctree {
        let mut svo = SparseVoxelOctree {
            node_pool: Vec::<Node>::new(),
            size: Vector3::new(1.0, 1.0, 1.0),
        };
        svo.node_pool.push(create_node());
        set_node(&mut svo.node_pool[0], false, false, 1, 0xFF00FF);
        svo.build_octree(&voxel_grid, 0, 0, 0, voxel_grid.size);
        return svo;
    }
}

impl SparseVoxelOctree {
    pub fn size_bytes(&self) -> usize {
        std::mem::size_of::<Node>() * self.node_pool.len()
    }
}
