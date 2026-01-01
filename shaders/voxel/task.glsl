#extension GL_EXT_mesh_shader : require
#extension GL_GOOGLE_include_directive : require
#extension GL_KHR_shader_subgroup_arithmetic : require
#include "../bindings.glsl"
#include "../voxel.glsl"

layout(local_size_x = 64) in;

struct VoxelPayload {
    uint meshlet_ids[64];
};

taskPayloadSharedEXT VoxelPayload payload;

void main() {
    uint meshlet_index = 64 * gl_WorkGroupID.x + gl_LocalInvocationIndex;
    VoxelMeshlet meshlet = voxel_meshlets[meshlet_index];

    bool cull = meshlet_index >= global.voxels.meshlet_count;
    cull = cull || back_cull(meshlet);
    cull = cull || frustum_cull(meshlet);

    uint task_count = subgroupAdd(cull ? 0 : 1);
    uint task_index = subgroupExclusiveAdd(cull ? 0 : 1);

    if (!cull) {
        payload.meshlet_ids[task_index] = meshlet_index;
    }
    EmitMeshTasksEXT(task_count, 1, 1);
}
