struct VoxelMeshlet {
    uint vertex_offset;
    uint vertex_count;
    uint triangle_offset;
    uint triangle_count;
    i16vec3 chunk;
};

struct VoxelVertex {
    u8vec3 position;
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
