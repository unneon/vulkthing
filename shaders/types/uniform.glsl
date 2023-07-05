// Vulkan GLSL forces you to use blocks as uniform types, with the intent being to discourage people from using multiple
// single-variable uniforms. However, I would like to reuse a big uniform struct across different shaders, hence the
// unpleasant macro.
#define GLOBAL_UNIFORM_TYPE \
    Global { \
        Grass grass; \
        Light light; \
        Settings settings; \
        Atmosphere atmosphere; \
        Gaussian gaussian; \
        Postprocessing postprocessing; \
        Camera camera; \
    }

struct Grass {
    float height_average;
    float height_max_variance;
    float width;
    float time;
    vec3 sway_direction;
    float sway_frequency;
    float sway_amplitude;
};

struct Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
    float diffuse_strength;
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
    vec3 position;
};

const uint TONEMAPPER_RGB_CLAMPING = 0;
const uint TONEMAPPER_REINHARD = 4;
const uint TONEMAPPER_NARKOWICZ_ACES = 8;
const uint TONEMAPPER_HILL_ACES = 9;
