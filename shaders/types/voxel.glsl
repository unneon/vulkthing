struct VoxelMeshlet {
    uint vertex_offset;
    uint vertex_count;
    uint triangle_offset;
    uint triangle_count;
    i16vec3 chunk;
    int16_t _pad0;
    u8vec3 bound_base;
    uint8_t _pad1;
    u8vec3 bound_size;
    uint8_t _pad2;
};

struct VoxelVertex {
    u8vec3 position;
    // Two lowest bits encode an ambient occlusion value (00 => 0, 11 => 3/4), the other bits are unused.
    uint8_t data;
};

struct VoxelTriangle {
    u8vec3 indices;
    // Three lowest bits encode a normal matching the convention of the DIRECTION array, five highest bits are used for
    // material ID. Subject to heavy changes later.
    uint8_t data;
};

struct VoxelPayload {
    uint meshlet_ids[64];
};

struct SparseVoxelOctree {
    // Either an octree index (31 lowest bits for index, 1 dicriminant 0) or an uniform voxel (5 lowest bits for
    // material id, 26 unused, 1 discriminant 1).
    uint material_or_pointer;
};
