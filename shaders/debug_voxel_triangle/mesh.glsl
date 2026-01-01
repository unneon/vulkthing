#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_mesh_shader : require
#include "../bindings.glsl"

// TODO: Mesa advertises 128 maxPreferredMeshWorkGroupInvocations contradicting their presentation and official drivers.
layout(local_size_x = 256) in;
layout(max_vertices = 128, max_primitives = 256) out;
layout(triangles) out;

void main() {
    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];
    SetMeshOutputsEXT(meshlet.vertex_count, meshlet.triangle_count);
    uint id = gl_LocalInvocationIndex;

    if (id < meshlet.triangle_count) {
        VoxelTriangle triangle = voxel_triangles[meshlet.triangle_offset + id];
        gl_PrimitiveTriangleIndicesEXT[id] = triangle.indices;
    }

    if (id < meshlet.vertex_count) {
        VoxelVertex vertex = voxel_vertices[meshlet.vertex_offset + id];
        vec3 world_space = vertex.position + global.voxels.chunk_size * vec3(meshlet.chunk);
        vec4 clip_space = global.camera.projection_matrix * global.camera.view_matrix * vec4(world_space, 1);
        gl_MeshVerticesEXT[id].gl_Position = clip_space;
    }
}

/*
pipeline "debug_voxel_triangle" mesh-shaders=true {
    cull-mode "NONE"
    polygon-mode "LINE"
}
*/
