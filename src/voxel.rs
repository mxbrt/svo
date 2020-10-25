use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Voxel {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub struct Grid {
    pub data: Vec<Vec<Vec<bool>>>,
    pub size: usize,
}

const BRICK_SIZE: usize = 8;
const BRICK_VOLUME: usize = BRICK_SIZE * BRICK_SIZE * BRICK_SIZE;
const BRICK_N_INTS: usize = BRICK_VOLUME / 32;

impl Grid {
    pub fn new(size: usize, voxels: &[Voxel]) -> Self {
        let mut data = vec![vec![vec![false; size]; size]; size];
        for voxel in voxels {
            data[voxel.x][voxel.y][voxel.z] = true;
        }
        Self { data, size }
    }

    pub fn sample(&self, x: usize, y: usize, z: usize, size: usize) -> bool {
        for x1 in x..x + size {
            for y1 in y..y + size {
                for z1 in z..z + size {
                    if self.data[x1][y1][z1] {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn brick_at(&self, x: usize, y: usize, z: usize) -> Option<[u32; BRICK_N_INTS]> {
        let mut data = [0; BRICK_N_INTS];
        let mut is_occupied = false;

        for i in 0..BRICK_SIZE.pow(3) {
            let x1 = i / BRICK_SIZE.pow(2);
            let y1 = (i / BRICK_SIZE) % BRICK_SIZE;
            let z1 = i % BRICK_SIZE;
            if self.data[x + x1][y + y1][z + z1] {
                let int_idx = i / 32;
                let bit_idx = i % 32;
                data[int_idx] |= 1 << bit_idx;
                is_occupied = true;
            }
        }

        match is_occupied {
            true => Some(data),
            false => None,
        }
    }
}

pub struct Brick {
    data: [u32; BRICK_N_INTS],
}

pub struct BrickPool {
    pub bricks: Vec<Brick>,
}

impl From<&Grid> for BrickPool {
    fn from(grid: &Grid) -> BrickPool {
        let mut bricks = Vec::new();
        for x in (0..grid.size).step_by(BRICK_SIZE) {
            for y in (0..grid.size).step_by(BRICK_SIZE) {
                for z in (0..grid.size).step_by(BRICK_SIZE) {
                    match grid.brick_at(x, y, z) {
                        Some(data) => bricks.push(Brick { data }),
                        None => (),
                    }
                }
            }
        }
        return BrickPool { bricks };
    }
}

impl BrickPool {
    fn occupancy(&self) -> f64 {
        let capacity = BRICK_VOLUME * self.bricks.len();
        let mut voxel_count = 0;
        for brick in &self.bricks {
            let brick_occupancy: u32 = brick.data.iter().map(|x| x.count_ones()).sum();
            voxel_count += brick_occupancy;
        }
        return (voxel_count as f64 / capacity as f64) * 100.0;
    }

    pub fn _print_stats(&self) {
        let size_kb = std::mem::size_of::<Brick>() * self.bricks.len() / 1024;
        println!("{} kB", size_kb);
        println!("{:.2}% brick occupancy", self.occupancy());
    }
}
