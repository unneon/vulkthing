#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

layout(binding = 2) uniform Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
    float diffuse_strength;
} light;

layout(binding = 3) uniform FragSettings {
    bool ray_traced_shadows;
} settings;

layout(binding = 4) uniform Atmosphere {
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

layout(binding = 5) uniform Camera {
    vec3 position;
} camera;

#ifdef SUPPORTS_RAYTRACING
layout(binding = 6) uniform accelerationStructureEXT tlas;
#endif

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec3 frag_ground_normal;
layout(location = 3) in float frag_naive_height;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/raytracing.glsl"

void main() {
    float height_factor = mix(0.7, 1, frag_naive_height);
    vec3 grass_color = vec3(0.2, 0.8, 0.03) * height_factor;
    float diffuse_factor = 1;

    vec3 ambient = light.ambient_strength * light.color * grass_color;
    vec3 diffuse = light.diffuse_strength * light.color * grass_color * diffuse_factor;
    if (in_shadow()) {
        diffuse = vec3(0);
    }

    vec3 color_at_object = ambient + diffuse;
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
