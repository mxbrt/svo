# Renderer for Sparse Voxel Octrees

This repository contains an experimental renderer for Sparse Voxel Octrees
(SVO). The renderer uses an adapted version of the raytracing algorithm
presented by Laine and Karras [1]. The following modifications were made:

- Voxels are rendered as cubes. Normal vectors are calculated on the fly and are not stored in the SVO.
- Contouring is not used


Multiple dynamic objects, each with their own transform, may be rendered.  To
achieve this, the renderer does a culling pass before raytracing.  This is
accelerated using a Bounding Volume Hierarchy.

# Examples

Stanford Dragon [2] at a voxel resolution of 512^3.

![dragon](/img/dragon.png?raw=true)

512 tori in a grid, each with a different rotation. The tori use a voxel
resolution of 128^3. In total, the scene contains over a billion voxels.
This scene is rendered at 20 FPS at 720p on an AMD 5700.

![torus](/img/torus.png?raw=true)

# Controls

WASD to move, mouse to look. Space to elevate camera, CTRL to descend.

# References

[1] https://research.nvidia.com/publication/efficient-sparse-voxel-octrees  
[2] http://graphics.stanford.edu/data/3Dscanrep
