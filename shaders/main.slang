import camera;
import geometry;
import global;
import voxel;

struct DebugVoxelScreenBoundVertex {
    float4 position : SV_Position;
};

[shader("mesh")]
[outputtopology("line")]
[numthreads(4, 1, 1)]
void debug_voxel_screen_bound_mesh(
    uint id : SV_GroupIndex,
    OutputVertices<DebugVoxelScreenBoundVertex, 4> vertices,
    OutputIndices<uint2, 4> indices,
) {
    const uint2 INDICES[4] = {
        uint2(0, 1),
        uint2(0, 2),
        uint2(1, 3),
        uint2(2, 3),
    };

    const float2 VERTICES[4] = {
        float2(0, 0),
        float2(0, 1),
        float2(1, 0),
        float2(1, 1),
    };

    VoxelMeshlet meshlet = meshlets[global.debug.meshlet_id];
    float3 world_space_min = global.voxels.chunk_size * float3(meshlet.chunk) + meshlet.bound_base;
    float3 world_space_max = world_space_min + meshlet.bound_size;
    float4 screen_aabb;
    if (screen_aabb_projection(world_space_min, world_space_max, global.camera.depth_near, mul(global.camera.projection_matrix, global.camera.view_matrix), screen_aabb)) {
        SetMeshOutputCounts(4, 4);
        indices[id] = INDICES[id];
        vertices[id].position = float4(screen_aabb.xy + (screen_aabb.zw - screen_aabb.xy) * VERTICES[id], 0, 1);
    } else {
        SetMeshOutputCounts(0, 0);
    }
}

[shader("fragment")]
float4 debug_voxel_screen_bound_frag() {
    return float4(113. / 255, 0. / 255, 0. / 255, 1);
}

[shader("fragment")]
float4 debug_voxel_triangle_frag() {
    return float4(238. / 255, 186. / 255, 11. / 255, 1);
}

[shader("fragment")]
float4 debug_voxel_world_bound_frag() {
    return float4(255. / 255, 78. / 255, 0. / 255, 1);
}

struct SkyboxVertex {
    float4 position : SV_Position;
    float3 direction : DIRECTION;
}

[shader("vertex")]
SkyboxVertex skybox_vert(uint id : SV_VertexID) {
    const float2 POSITIONS[6] = {
        float2(1, 1),
        float2(1, -1),
        float2(-1, -1),
        float2(1, 1),
        float2(-1, -1),
        float2(-1, 1)
    };

    float4 normalized_clip_space = float4(POSITIONS[id], 1, 1);
    float3 direction = normalize(mul(global.camera.inverse_view_matrix, mul(global.camera.inverse_projection_matrix, normalized_clip_space)).xyz);

    SkyboxVertex vertex;
    vertex.position = normalized_clip_space;
    vertex.direction = direction;
    return vertex;
}

[shader("fragment")]
float4 skybox_frag() {
    return float4(float3(0), 1);
}

[shader("vertex")]
float4 star_vert(uint instance_id : SV_InstanceID, float3 in_position : POSITION) : SV_Position {
    float4x4 star_model = stars[instance_id].model;
    float4 world_space = mul(star_model, float4(in_position, 1));
    float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, world_space));
    return clip_space;
}

[shader("fragment")]
float4 star_frag() {
    return float4(float3(10), 1);
}

[shader("vertex")]
float4 sun_vert(float3 position) : SV_Position {
    float4 world_space = float4(50 * position + global.light.position, 1);
    float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, world_space));
    return clip_space;
}

[shader("fragment")]
float4 sun_frag(){
    return float4(float3(100), 1);
}

[shader("vertex")]
float4 voxel_rt_vert(uint id : SV_VertexID) : SV_Position {
    const float2 POSITIONS[6] = {
        float2(1, 1),
        float2(1, -1),
        float2(-1, -1),
        float2(1, 1),
        float2(-1, -1),
        float2(-1, 1)
    };

    return float4(POSITIONS[id], 0.9, 1);
}

[shader("fragment")]
float4 voxel_rt_frag(float4 frag_coord : SV_Position) {
    float3 camera_position_within_cube = fmod(global.camera.position, 1);
    float3 view_direction = normalize(world_space_from_depth(1, frag_coord));
    uint3 voxel = ray_trace(int3(floor(global.camera.position)), camera_position_within_cube, view_direction);
    uint material_index = find_svo(voxel);
    VoxelMaterial material = global.materials[material_index];
    float3 color = material.albedo;
    return float4(color, 1);
}
