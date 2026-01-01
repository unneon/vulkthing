#ifndef BINDINGS_GLSL
#define BINDINGS_GLSL

#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_shader_explicit_arithmetic_types : require
#extension GL_EXT_shader_explicit_arithmetic_types_int8 : require
#extension GL_EXT_shader_explicit_arithmetic_types_int16 : require

struct Voxels {
    uint chunk_size;
    uint meshlet_count;
    uint root_svo_index;
    uint root_svo_side;
    uvec3 root_svo_base;
};

struct Light {
    vec3 color;
    float intensity;
    vec3 position;
    float scale;
};

struct Atmosphere {
    bool enable;
    uint scatter_point_count;
    uint optical_depth_point_count;
    float density_falloff;
    vec3 planet_position;
    float planet_radius;
    vec3 sun_position;
    float scale;
    vec3 wavelengths;
    float scattering_strength;
    float henyey_greenstein_g;
};

struct Camera {
    mat4 view_matrix;
    mat4 projection_matrix;
    mat4 inverse_view_matrix;
    mat4 inverse_projection_matrix;
    vec2 resolution;
    float depth_near;
    float depth_far;
    vec3 position;
    vec3 direction;
};

struct VoxelMaterial {
    vec3 albedo;
    float roughness;
    vec3 emit;
    float metallic;
};

struct Debug {
    uint meshlet_id;
};

struct Star {
    mat4 model;
};

struct VoxelVertex {
    u8vec3 position;
    uint8_t data;
};

float voxel_vertex_ambient_occlusion(VoxelVertex v) {
    return 0.75 * float(bitfieldExtract(v.data, 0, 2)) / 3;
}

struct VoxelTriangleData {
    uint8_t data;
};

uint8_t voxel_triangle_cubenormal(VoxelTriangleData d) {
    return uint8_t(bitfieldExtract(d.data, 0, 3));
}

uint8_t voxel_triangle_material_id(VoxelTriangleData triangle) {
    return uint8_t(bitfieldExtract(triangle.data, 3, 5));
}

struct VoxelTriangle {
    u8vec3 indices;
    VoxelTriangleData data;
};

struct VoxelMeshlet {
    uint vertex_offset;
    uint vertex_count;
    uint triangle_offset;
    uint triangle_count;
    i16vec3 chunk;
    u8vec3 bound_base;
    u8vec3 bound_size;
};

struct SvoNode {
    uint children[8];
    uint parent;
};

struct ClassicVertex {
    vec3 position;
    vec3 normal;
};

layout(scalar, set = 0, binding = 0) uniform Global {
    Voxels voxels;
    Light light;
    Atmosphere atmosphere;
    Camera camera;
    VoxelMaterial materials[256];
    Debug debug;
} global;

layout(scalar, set = 0, binding = 1) readonly buffer Stars {
    Star stars[];
};

layout(scalar, set = 0, binding = 2) readonly buffer VoxelVertices {
    VoxelVertex voxel_vertices[];
};

layout(scalar, set = 0, binding = 3) readonly buffer VoxelTriangles {
    VoxelTriangle voxel_triangles[];
};

layout(scalar, set = 0, binding = 4) readonly buffer VoxelMeshlets {
    VoxelMeshlet voxel_meshlets[];
};

layout(scalar, set = 0, binding = 5) readonly buffer SvoNodes {
    SvoNode svo_nodes[];
};

layout(scalar, set = 0, binding = 6) readonly buffer ClassicVertices {
    ClassicVertex classic_vertices[];
};

vec3 voxel_meshlet_world_space_min(VoxelMeshlet m) {
    return global.voxels.chunk_size * vec3(m.chunk) + m.bound_base;
}

vec3 voxel_meshlet_world_space_max(VoxelMeshlet m) {
    return voxel_meshlet_world_space_min(m) + vec3(m.bound_size);
}

#endif
