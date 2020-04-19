use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3};

use crate::render::{RaycastResult, Renderable};
use crate::voxel_grid::VoxelGrid;

#[derive(Clone, Copy)]
pub struct Node {
    pub children: u32,
    pub color: u32,
}

impl Node {
    fn new() -> Node {
        Node {
            children: 1 << 31, // default is a terminal node
            color: 0,
        }
    }

    fn set(&mut self, is_terminal: bool, is_leaf: bool, children: usize, color: u32) {
        self.children = ((is_terminal as u32) << 31)
            | ((is_leaf as u32) << 30)
            | (children as u32 & 0x3fffffff);
        self.color = color;
    }

    fn is_terminal(&self) -> bool {
        (self.children & 0x80000000) != 0
    }

    fn is_leaf(&self) -> bool {
        (self.children & 0x40000000) != 0
    }

    fn child_idx(&self) -> usize {
        (self.children & 0x3fffffff) as usize
    }
}

#[derive(Clone)]
pub struct SparseVoxelOctree {
    pub node_pool: Vec<Node>,
    pub size: Vector3<f32>,
    pub model: Matrix4<f32>,
}

struct RayData {
    hit_idx: usize,
    visit_cnt: u32,
    a: usize,
    hit_pos: Point3<f32>,
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
            ([x, x, x, x, x + half_size, x + half_size, x + half_size, x + half_size],
             [y, y, y + half_size, y + half_size, y, y, y + half_size, y + half_size],
             [z, z + half_size, z, z + half_size, z, z + half_size, z, z + half_size]);

        let node_tile_idx = self.node_pool.len();
        self.node_pool.append(&mut vec![Node::new(); 8]);

        for i in 0..8 {
            if voxel_grid.sample(cx[i], cy[i], cz[i], half_size) {
                let color = (cx[i] | (cy[i] << 8) | (cz[i] << 16)) as u32;
                if half_size != 1 {
                    let child_idx = self.build_octree(voxel_grid, cx[i], cy[i], cz[i], half_size);
                    self.node_pool[node_tile_idx + i].set(false, false, child_idx, color);
                } else {
                    self.node_pool[node_tile_idx + i].set(false, true, 0x3fffffff, color);
                }
            }
        }
        return node_tile_idx;
    }

    fn new_node(&self, txm: f32, x: usize, tym: f32, y: usize, tzm: f32, z: usize) -> usize {
        if txm < tym {
            // YZ plane
            if txm < tzm {
                return x;
            }
        } else {
            // XZ plane
            if tym < tzm {
                return y;
            }
        }
        return z; // XY plane;
    }

    fn first_node(&self, tx0: f32, ty0: f32, tz0: f32, txm: f32, tym: f32, tzm: f32) -> usize {
        let mut idx = 0;
        if tx0 > ty0 {
            if tx0 > tz0 {
                if tym < tx0 {
                    idx |= 2;
                }
                if tzm < tx0 {
                    idx |= 1;
                }
                return idx;
            }
        } else {
            if ty0 > tz0 {
                if txm < ty0 {
                    idx |= 4;
                }
                if tzm < ty0 {
                    idx |= 1;
                }
                return idx;
            }
        }
        if txm < tz0 {
            idx |= 4;
        }
        if tym < tz0 {
            idx |= 2;
        }
        return idx;
    }

    fn proc_subtree(
        &self,
        tx0: f32,
        ty0: f32,
        tz0: f32,
        tx1: f32,
        ty1: f32,
        tz1: f32,
        idx: usize,
        ray_data: &mut RayData,
    ) -> bool {
        if tx1 < 0.0 || ty1 < 0.0 || tz1 < 0.0 {
            return false;
        }

        let node = self.node_pool[idx];
        let children_idx = node.child_idx();
        ray_data.visit_cnt += 1;

        if node.is_leaf() {
            ray_data.hit_idx = idx;
            ray_data.hit_pos.x = tx0;
            ray_data.hit_pos.y = ty0;
            ray_data.hit_pos.z = tz0;
            return true;
        }

        if node.is_terminal() {
            return false;
        }

        let txm = 0.5 * (tx0 + tx1);
        let tym = 0.5 * (ty0 + ty1);
        let tzm = 0.5 * (tz0 + tz1);

        let mut cur_node = self.first_node(tx0, ty0, tz0, txm, tym, tzm);
        let mut is_hit = false;
        while !is_hit {
            let next = children_idx + (cur_node ^ ray_data.a);
            match cur_node {
                0 => {
                    is_hit = self.proc_subtree(tx0, ty0, tz0, txm, tym, tzm, next, ray_data);
                    cur_node = self.new_node(txm, 4, tym, 2, tzm, 1);
                }
                1 => {
                    is_hit = self.proc_subtree(tx0, ty0, tzm, txm, tym, tz1, next, ray_data);
                    cur_node = self.new_node(txm, 5, tym, 3, tz1, 8);
                }
                2 => {
                    is_hit = self.proc_subtree(tx0, tym, tz0, txm, ty1, tzm, next, ray_data);
                    cur_node = self.new_node(txm, 6, ty1, 8, tzm, 3);
                }
                3 => {
                    is_hit = self.proc_subtree(tx0, tym, tzm, txm, ty1, tz1, next, ray_data);
                    cur_node = self.new_node(txm, 7, ty1, 8, tz1, 8);
                }
                4 => {
                    is_hit = self.proc_subtree(txm, ty0, tz0, tx1, tym, tzm, next, ray_data);
                    cur_node = self.new_node(tx1, 8, tym, 6, tzm, 5);
                }
                5 => {
                    is_hit = self.proc_subtree(txm, ty0, tzm, tx1, tym, tz1, next, ray_data);
                    cur_node = self.new_node(tx1, 8, tym, 7, tz1, 8);
                }
                6 => {
                    is_hit = self.proc_subtree(txm, tym, tz0, tx1, ty1, tzm, next, ray_data);
                    cur_node = self.new_node(tx1, 8, ty1, 8, tzm, 7);
                }
                7 => {
                    is_hit = self.proc_subtree(txm, tym, tzm, tx1, ty1, tz1, next, ray_data);
                    cur_node = 8;
                }
                _ => {
                    return false;
                }
            }
        }
        return true;
    }
}

impl Renderable for SparseVoxelOctree {
    fn raycast(&self, origin: Point3<f32>, dir: Vector3<f32>, result: &mut RaycastResult) -> bool {
        let mut ray_data = RayData {
            hit_idx: 0,
            visit_cnt: 0,
            a: 0,
            hit_pos: Point3::origin(),
        };
        let mut origin = self.model.transform_point(origin);
        let mut dir = self.model.transform_vector(dir);
        if dir.x < 0.0 {
            origin.x = self.size.x - origin.x;
            dir.x = -dir.x;
            ray_data.a |= 4;
        }
        if dir.y < 0.0 {
            origin.y = self.size.y - origin.y;
            dir.y = -dir.y;
            ray_data.a |= 2;
        }
        if dir.z < 0.0 {
            origin.z = self.size.z - origin.z;
            dir.z = -dir.z;
            ray_data.a |= 1;
        }

        let tx0 = -origin.x / dir.x;
        let ty0 = -origin.y / dir.y;
        let tz0 = -origin.z / dir.z;
        let tx1 = (self.size.x - origin.x) / dir.x;
        let ty1 = (self.size.y - origin.y) / dir.y;
        let tz1 = (self.size.z - origin.z) / dir.z;

        if f32::max(f32::max(tx0, ty0), tz0) < f32::min(f32::min(tx1, ty1), tz1) {
            if self.proc_subtree(tx0, ty0, tz0, tx1, ty1, tz1, 0, &mut ray_data) {
                result.visit_cnt = ray_data.visit_cnt;
                result.color = self.node_pool[ray_data.hit_idx].color;
                let tx0 = ray_data.hit_pos.x;
                let ty0 = ray_data.hit_pos.y;
                let tz0 = ray_data.hit_pos.z;
                let face;
                if tx0 > ty0 && tx0 > tz0 {
                    face = 0 | (ray_data.a & 4);
                } else if ty0 > tx0 && ty0 > tz0 {
                    face = 1 | (ray_data.a & 2);
                } else {
                    face = 2 | ((ray_data.a & 1) << 2);
                }
                match face {
                    // left
                    0 => result.normal.x = -1.0,
                    // bottom
                    1 => result.normal.y = -1.0,
                    // front
                    2 => result.normal.z = 1.0,
                    // top
                    3 => result.normal.y = 1.0,
                    // right side
                    4 => result.normal.x = 1.0,
                    // back
                    6 => result.normal.z = -1.0,
                    _ => println!("Invalid value for cube face."),
                }
                result.normal = self.model.transform_vector(result.normal);
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    fn get_model(&self) -> &Matrix4<f32> {
        return &self.model;
    }
}

impl From<&VoxelGrid> for SparseVoxelOctree {
    fn from(voxel_grid: &VoxelGrid) -> SparseVoxelOctree {
        let mut svo = SparseVoxelOctree {
            node_pool: Vec::<Node>::new(),
            size: Vector3::new(1.0, 1.0, 1.0),
            model: Matrix4::from_translation(Vector3::zero()),
        };
        svo.node_pool.push(Node::new());
        svo.node_pool[0].set(false, false, 1, 0);
        svo.build_octree(&voxel_grid, 0, 0, 0, voxel_grid.size);
        return svo;
    }
}
