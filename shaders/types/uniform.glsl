// Vulkan GLSL forces you to use blocks as uniform types, with the intent being to discourage people from using multiple
// single-variable uniforms. However, I would like to reuse a big uniform struct across different shaders, hence the
// unpleasant macros.

#define GLOBAL_UNIFORM_TYPE \
    Global { \
        Light light; \
        Atmosphere atmosphere; \
        Postprocessing postprocessing; \
        Camera camera; \
        VoxelMaterial materials[256]; \
    }

#define MATERIAL_UNIFORM_TYPE \
    Material { \
        vec3 albedo; \
        float metallic; \
        vec3 emit; \
        float roughness; \
    }

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

struct Postprocessing {
    float exposure;
    uint tonemapper;
    float gamma;
};

struct Camera {
    mat4 view_matrix;
    mat4 projection_matrix;
    mat4 inverse_view_matrix;
    mat4 inverse_projection_matrix;
    vec2 resolution;
    vec3 position;
};

struct VoxelVertex {
    u8vec3 position;
};

struct VoxelTriangle {
    u8vec3 indices;
    // Three lowest bits encode a normal matching the convention of the DIRECTION array, five highest bits are used for
    // material ID. Subject to heavy changes later.
    uint8_t data;
};

struct VoxelMeshlet {
    uint vertex_offset;
    uint vertex_count;
    uint triangle_offset;
    uint triangle_count;
    i16vec3 chunk;
};

struct VoxelMaterial {
    vec3 albedo;
    float roughness;
    vec3 emit;
    float metallic;
};

struct Star {
    mat4 model;
};

const uint TONEMAPPER_RGB_CLAMPING = 0;
const uint TONEMAPPER_REINHARD = 4;
const uint TONEMAPPER_NARKOWICZ_ACES = 8;
const uint TONEMAPPER_HILL_ACES = 9;
