// Vulkan GLSL forces you to use blocks as uniform types, with the intent being to discourage people from using multiple
// single-variable uniforms. However, I would like to reuse a big uniform struct across different shaders, hence the
// unpleasant macros.

#define GLOBAL_UNIFORM_TYPE \
    Global { \
        Light light; \
        Settings settings; \
        Atmosphere atmosphere; \
        Gaussian gaussian; \
        Postprocessing postprocessing; \
        Camera camera; \
    }

#define MATERIAL_UNIFORM_TYPE \
    Material { \
        vec3 albedo; \
        float metallic; \
        vec3 emit; \
        float roughness; \
        float ao; \
    }

struct Light {
    vec3 color;
    float intensity;
    vec3 position;
    float shadow_sample_seed;
    float scale;
};

struct Settings {
    bool ray_traced_shadows;
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

struct Gaussian {
    float threshold;
    int radius;
    float exponent_coefficient;
};

struct Postprocessing {
    vec3 color_filter;
    float bloom;
    float exposure;
    float temperature;
    float tint;
    float contrast;
    float brightness;
    float saturation;
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

const uint TONEMAPPER_RGB_CLAMPING = 0;
const uint TONEMAPPER_REINHARD = 4;
const uint TONEMAPPER_NARKOWICZ_ACES = 8;
const uint TONEMAPPER_HILL_ACES = 9;
