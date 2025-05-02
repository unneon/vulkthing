uniform Global global;

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

[shader("fragment")]
float4 skybox_frag(float3 frag_direction) {
    return float4(0, 0, 0, 1);
}

[shader("fragment")]
float4 star_frag(float3 frag_position) {
    float3 color_at_object = float3(10);
    return float4(color_at_object, 1);
}

[shader("fragment")]
float4 sun_frag(float3 frag_position) {
    float3 color_at_object = float3(100);
    return float4(color_at_object, 1);
}
