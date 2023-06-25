#version 460

layout(binding = 1) uniform Atmosphere {
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
} atmosphere;

layout(binding = 2) uniform Camera {
    vec3 position;
} camera;

layout(location = 0) in vec3 frag_position;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"

void main() {
    out_color = vec4(compute_atmosphere(vec3(0), frag_position), 1);
}
