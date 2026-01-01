#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_mesh_shader : require
#include "../bindings.glsl"

layout(local_size_x = 12) in;
layout(max_vertices = 8, max_primitives = 12) out;
layout(lines) out;

const uvec2 INDICES[12] = { { 0, 1 }, { 0, 2 }, { 1, 3 }, { 2, 3 }, { 0, 4 }, { 1, 5 }, { 2, 6 }, { 3, 7 }, { 4, 5 }, { 4, 6 }, { 5, 7 }, { 6, 7 } };
const vec3 VERTICES[8] = { { 0, 0, 0 }, { 0, 1, 0 }, { 1, 0, 0 }, { 1, 1, 0 }, { 0, 0, 1 }, { 0, 1, 1 }, { 1, 0, 1 }, { 1, 1, 1 } };

void main() {
    SetMeshOutputsEXT(8, 12);
    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];
    uint id = gl_LocalInvocationIndex;

    gl_PrimitiveLineIndicesEXT[id] = INDICES[id];

    if (id < 8) {
        vec3 world_space = global.voxels.chunk_size * meshlet.chunk + meshlet.bound_base + VERTICES[id] * vec3(meshlet.bound_size.x, meshlet.bound_size.y, meshlet.bound_size.z);
        vec4 clip_space = global.camera.projection_matrix * global.camera.view_matrix * vec4(world_space, 1);
        gl_MeshVerticesEXT[id].gl_Position = clip_space;
    }
}