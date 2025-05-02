[[vk::binding(0)]]
uniform Global global;

[[vk::binding(1)]]
StructuredBuffer<Star> stars;

struct Global {
    Voxels voxels;
    Light light;
    Atmosphere atmosphere;
    Postprocessing postprocessing;
    Camera camera;
    VoxelMaterial materials[256];
    Debug debug;
};

struct Voxels {
    uint chunk_size;
    uint meshlet_count;
    uint root_svo_index;
    uint root_svo_side;
    uint3 root_svo_base;
};

struct Light {
    float3 color;
    float intensity;
    float3 position;
    float scale;
};

struct Atmosphere {
    bool enable;
    uint scatter_point_count;
    uint optical_depth_point_count;
    float density_falloff;
    float3 planet_position;
    float planet_radius;
    float3 sun_position;
    float scale;
    float3 wavelengths;
    float scattering_strength;
    float henyey_greenstein_g;
};

struct Postprocessing {
    float exposure;
    uint tonemapper;
    float gamma;
};

struct Camera {
    float4x4 view_matrix;
    float4x4 projection_matrix;
    float4x4 inverse_view_matrix;
    float4x4 inverse_projection_matrix;
    float2 resolution;
    float depth_near;
    float depth_far;
    float3 position;
    float3 direction;
};

struct VoxelMaterial {
    float3 albedo;
    float roughness;
    float3 emit;
    float metallic;
};

struct Debug {
    uint meshlet_id;
};

struct Star {
    float4x4 model;
};

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
