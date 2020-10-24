// TODO: use assembly intrinsics

pub fn encode_3d(x: u64, y: u64, z: u64) -> u64 {
    bloat_2(x) | (bloat_2(y) << 1) | (bloat_2(z) << 2)
}

pub fn decode_3d(x: u64) -> (u64, u64, u64) {
    (shrink_2(x), shrink_2(x >> 1), shrink_2(x >> 2))
}

// "Insert" two 0 bits after each of the 21 low bits of x
fn bloat_2(mut x: u64) -> u64 {
    x = x & 0x1fffff;
    x = (x | (x << 32)) & 0x1f00000000ffff;
    x = (x | (x << 16)) & 0x1f0000ff0000ff;
    x = (x | (x << 8)) & 0x100f00f00f00f00f;
    x = (x | (x << 4)) & 0x10c30c30c30c30c3;
    (x | x << 2) & 0x1249249249249249
}

// reverse bloat_2
fn shrink_2(mut x: u64) -> u64 {
    x = x & 0x1249249249249249;
    x = (x ^ (x >> 2)) & 0x10c30c30c30c30c3;
    x = (x ^ (x >> 4)) & 0x100f00f00f00f00f;
    x = (x ^ (x >> 8)) & 0x1f0000ff0000ff;
    x = (x ^ (x >> 16)) & 0x1f00000000ffff;
    (x ^ (x >> 32)) & 0x1fffff
}

#[test]
fn test_morton() {
    let n = 128;
    for x in 0..n {
        for y in 0..n {
            for z in 0..n {
                let m = encode_3d(x, y, z);
                let (x1, y1, z1) = decode_3d(m);
                assert!(x == x1 && y == y1 && z == z1);
            }
        }
    }
}
