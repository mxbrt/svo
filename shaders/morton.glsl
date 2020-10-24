#extension GL_ARB_gpu_shader_int64: require

uint64_t bloat2(uint64_t x) {
    x = x & 0x1fffffUL;
    x = (x | (x << 32)) & 0x1f00000000ffffUL;
    x = (x | (x << 16)) & 0x1f0000ff0000ffUL;
    x = (x | (x << 8)) & 0x100f00f00f00f00fUL;
    x = (x | (x << 4)) & 0x10c30c30c30c30c3UL;
    x = (x | x << 2) & 0x1249249249249249UL;
    return x;
}

uint64_t shrink2(uint64_t x) {
    x = x & 0x1249249249249249UL;
    x = (x ^ (x >> 2)) & 0x10c30c30c30c30c3UL;
    x = (x ^ (x >> 4)) & 0x100f00f00f00f00fUL;
    x = (x ^ (x >> 8)) & 0x1f0000ff0000ffUL;
    x = (x ^ (x >> 16)) & 0x1f00000000ffffUL;
    x = (x ^ (x >> 32)) & 0x1fffffUL;
    return x;
}

uint64_t encode3d(ivec3 coords) {
    return bloat2(coords.x) | (bloat2(coords.y) << 1) | (bloat2(coords.z) << 2);
}

ivec3 decode3d(uint64_t x) {
    return ivec3(shrink2(x), shrink2(x >> 1), shrink2(x >> 2));
}
