#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_mesh_shader : require
#include "../bindings.glsl"
#include "../geometry.glsl"

layout(local_size_x = 4) in;
layout(max_vertices = 4, max_primitives = 4) out;
layout(lines) out;

const uvec2 INDICES[4] = { { 0, 1 }, { 0, 2 }, { 1, 3 }, { 2, 3 } };
const vec2 VERTICES[4] = { { 0, 0 }, { 0, 1 }, { 1, 0 }, { 1, 1 } };

void main() {
    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];
    uint id = gl_LocalInvocationIndex;

    vec4 screen_aabb;
    if (screen_aabb_projection(voxel_meshlet_world_space_min(meshlet), voxel_meshlet_world_space_max(meshlet), screen_aabb)) {
        SetMeshOutputsEXT(4, 4);
        gl_PrimitiveLineIndicesEXT[id] = INDICES[id];
        gl_MeshVerticesEXT[id].gl_Position = vec4(screen_aabb.xy + (screen_aabb.zw - screen_aabb.xy) * VERTICES[id], 0, 1);
    } else {
        SetMeshOutputsEXT(0, 0);
    }
}