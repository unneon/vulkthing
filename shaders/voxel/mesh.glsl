#extension GL_EXT_mesh_shader : require
#extension GL_GOOGLE_include_directive : require
#include "../bindings.glsl"

struct VoxelPayload {
    uint meshlet_ids[64];
};

// TODO: Mesa advertises 128 maxPreferredMeshWorkGroupInvocations contradicting their presentation and official drivers.
layout(local_size_x = 256) in;
layout(triangles, max_vertices = 128, max_primitives = 256) out;

layout(location = 0) out float ambient_occlusion[];
layout(location = 1) perprimitiveEXT out uint triangle_data[];

taskPayloadSharedEXT VoxelPayload payload;

void main() {
    VoxelMeshlet meshlet = voxel_meshlets[payload.meshlet_ids[gl_WorkGroupID.x]];
    SetMeshOutputsEXT(meshlet.vertex_count, meshlet.triangle_count);

    if (gl_LocalInvocationIndex < meshlet.triangle_count) {
        VoxelTriangle triangle = voxel_triangles[meshlet.triangle_offset + gl_LocalInvocationIndex];
        gl_PrimitiveTriangleIndicesEXT[gl_LocalInvocationIndex] = triangle.indices;
        triangle_data[gl_LocalInvocationIndex] = triangle.data.data;
    }

    if (gl_LocalInvocationIndex < meshlet.vertex_count) {
        VoxelVertex vertex = voxel_vertices[meshlet.vertex_offset + gl_LocalInvocationID.x];
        vec3 world_space = vertex.position + global.voxels.chunk_size * vec3(meshlet.chunk);
        vec4 clip_space = global.camera.projection_matrix * global.camera.view_matrix * vec4(world_space, 1);
        gl_MeshVerticesEXT[gl_LocalInvocationID.x].gl_Position = clip_space;
        ambient_occlusion[gl_LocalInvocationID.x] = 0.75 * float(vertex.data & 3) / 3;
    }
}
