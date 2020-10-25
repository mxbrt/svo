use crate::voxel;

pub fn parse_size(path: &String) -> usize {
    let size_start = path
        .find("_")
        .expect(&format!("Filename missing '_': {}", path))
        + 1;
    let size_end = path
        .find(".")
        .expect(&format!("Filename missing '.': {}", path));
    let size_str = &path[size_start..size_end];
    size_str.parse::<usize>().expect(&format!(
        "Failed to parse integer: {} in {}",
        size_str, path
    ))
}

pub fn read(path: &String) -> Vec<voxel::Voxel> {
    let mut voxels = Vec::new();
    let mut csv_reader = csv::Reader::from_path(&path).unwrap();
    for result in csv_reader.deserialize() {
        let voxel: voxel::Voxel = match result {
            Ok(voxel) => voxel,
            Err(e) => panic!("{}: csv parsing error: {}", path, e),
        };
        voxels.push(voxel);
    }
    voxels
}
