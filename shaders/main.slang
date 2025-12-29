import bindings;
import camera;
import geometry;
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
    const uint2 INDICES[4] = {{0, 1}, {0, 2}, {1, 3}, {2, 3}};
    const float2 VERTICES[4] = {{0, 0}, {0, 1}, {1, 0}, {1, 1}};

    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];
    if (let screen_aabb = screen_aabb_projection(meshlet.world_space_min, meshlet.world_space_max)) {
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

struct DebugVoxelTriangleVertex {
    float4 position : SV_Position;
};

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(256, 1, 1)]
// TODO: Mesa advertises 128 maxPreferredMeshWorkGroupInvocations contradicting their presentation and official drivers.
void debug_voxel_triangle_mesh(
    uint id : SV_GroupIndex,
    OutputVertices<DebugVoxelTriangleVertex, 128> vertices,
    OutputIndices<uint3, 256> indices
) {
    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];
    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (id < meshlet.triangle_count) {
        VoxelTriangle triangle = voxel_triangles[meshlet.triangle_offset + id];
        indices[id] = triangle.indices;
    }

    if (id < meshlet.vertex_count) {
        VoxelVertex vertex = voxel_vertices[meshlet.vertex_offset + id];
        float3 world_space = vertex.position + global.voxels.chunk_size * float3(meshlet.chunk);
        float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, float4(world_space, 1)));
        vertices[id].position = clip_space;
    }
}

[shader("fragment")]
float4 debug_voxel_triangle_frag() {
    return float4(238. / 255, 186. / 255, 11. / 255, 1);
}

struct DebugVoxelWorldBoundVertex {
    float4 position : SV_Position;
};

[shader("mesh")]
[outputtopology("line")]
[numthreads(12, 1, 1)]
void debug_voxel_world_bound_mesh(
    uint id : SV_GroupIndex,
    OutputVertices<DebugVoxelWorldBoundVertex, 8> vertices,
    OutputIndices<uint2, 12> indices,
) {
    const uint2 INDICES[12] = {{0, 1}, {0, 2}, {1, 3}, {2, 3}, {0, 4}, {1, 5}, {2, 6}, {3, 7}, {4, 5}, {4, 6}, {5, 7}, {6, 7}};
    const float3 VERTICES[8] = {{0, 0, 0}, {0, 1, 0}, {1, 0, 0}, {1, 1, 0}, {0, 0, 1}, {0, 1, 1}, {1, 0, 1}, {1, 1, 1}};

    SetMeshOutputCounts(8, 12);
    VoxelMeshlet meshlet = voxel_meshlets[global.debug.meshlet_id];

    indices[id] = INDICES[id];

    if (id < 8) {
        float3 world_space = global.voxels.chunk_size * float3(meshlet.chunk) + meshlet.bound_base + VERTICES[id] * meshlet.bound_size;
        float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, float4(world_space, 1)));
        vertices[id].position = clip_space;
    }
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
    const float2 POSITIONS[6] = {{1, 1}, {1, -1}, {-1, -1}, {1, 1}, {-1, -1}, {-1, 1}};

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
float4 star_vert(uint instance_id : SV_InstanceID, uint vertex_base : SV_StartVertexLocation, uint vertex_id : SV_VertexID) : SV_Position {
    float3 in_position = classic_vertices[vertex_base + vertex_id].position;
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
float4 sun_vert(uint vertex_base : SV_StartVertexLocation, uint vertex_id : SV_VertexID) : SV_Position {
    float3 position = classic_vertices[vertex_base + vertex_id].position;
    float4 world_space = float4(50 * position + global.light.position, 1);
    float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, world_space));
    return clip_space;
}

[shader("fragment")]
float4 sun_frag(){
    return float4(float3(100), 1);
}

struct VoxelPayload {
    uint meshlet_ids[64];
};

struct VoxelMeshVertex {
    float4 position : SV_Position;
    float ambient_occlusion : AMBIENT_OCCLUSION;
};

struct VoxelMeshTriangle {
    uint data : TRIANGLE_DATA;
};

groupshared VoxelPayload voxel_task_payload;

[shader("amplification")]
[numthreads(64, 1, 1)]
void voxel_task(
    uint group_id : SV_GroupID,
    uint id : SV_GroupIndex,
) {
    uint meshlet_index = 64 * group_id.x + id.x;
    VoxelMeshlet meshlet = voxel_meshlets[meshlet_index];

    bool cull = meshlet_index >= global.voxels.meshlet_count;
    cull = cull || back_cull(meshlet);
    cull = cull || frustum_cull(meshlet);

    uint task_count = WaveActiveSum(cull ? 0 : 1);
    uint task_index = WavePrefixSum(cull ? 0 : 1);

    if (!cull) {
        voxel_task_payload.meshlet_ids[task_index] = meshlet_index;
    }
    DispatchMesh(task_count, 1, 1, voxel_task_payload);
}

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(256, 1, 1)]
// TODO: Mesa advertises 128 maxPreferredMeshWorkGroupInvocations contradicting their presentation and official drivers.
void voxel_mesh(
    uint group_id : SV_GroupID,
    uint id : SV_GroupIndex,
    in payload VoxelPayload payload,
    OutputVertices<VoxelMeshVertex, 128> vertices,
    OutputIndices<uint3, 256> indices,
    OutputPrimitives<VoxelMeshTriangle, 256> primitives,
) {
    VoxelMeshlet meshlet = voxel_meshlets[payload.meshlet_ids[group_id.x]];
    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (id < meshlet.triangle_count) {
        VoxelTriangle triangle = voxel_triangles[meshlet.triangle_offset + id];
        indices[id] = triangle.indices;
        primitives[id].data = triangle.data.data;
    }

    if (id < meshlet.vertex_count) {
        VoxelVertex vertex = voxel_vertices[meshlet.vertex_offset + id];
        float3 world_space = vertex.position + global.voxels.chunk_size * float3(meshlet.chunk);
        float4 clip_space = mul(global.camera.projection_matrix, mul(global.camera.view_matrix, float4(world_space, 1)));
        vertices[id].position = clip_space;
        vertices[id].ambient_occlusion = vertex.ambient_occlusion;
    }
}

[shader("fragment")]
float4 voxel_frag(
    float ambient_occlusion : AMBIENT_OCCLUSION,
    uint triangle_data : TRIANGLE_DATA,
) {
    VoxelTriangleData triangle_data = {triangle_data};
    VoxelMaterial material = global.materials[triangle_data.material_id];
    float3 light = (1 - ambient_occlusion) * material.albedo;
    return float4(light, 1);
}

[shader("vertex")]
float4 voxel_rt_vert(uint id : SV_VertexID) : SV_Position {
    const float2 POSITIONS[6] = {{1, 1}, {1, -1}, {-1, -1}, {1, 1}, {-1, -1}, {-1, 1}};

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
