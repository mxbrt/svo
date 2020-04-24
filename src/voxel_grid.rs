use cgmath::Point3;

pub struct VoxelGrid {
    pub data: Vec<Vec<Vec<bool>>>,
    pub size: usize,
}

impl VoxelGrid {
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

    pub fn from_csv(path: String) -> Result<VoxelGrid, Box<dyn std::error::Error>> {
        let mut csv_reader = csv::Reader::from_path(&path)?;
        let size_start = path.find("_").unwrap() + 1;
        let size_end = path.find(".").unwrap();
        let size_str = &path[size_start..size_end];
        let size = size_str.parse::<usize>().unwrap();
        let mut data = vec![vec![vec![false; size]; size]; size];
        for result in csv_reader.deserialize() {
            let coords: Point3<usize> = result?;
            data[coords.x][coords.y][coords.z] = true;
        }
        Ok(VoxelGrid {
            data: data,
            size: size,
        })
    }
}
