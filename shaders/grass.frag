#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

#include "types/uniform.glsl"

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;
#ifdef SUPPORTS_RAYTRACING
layout(binding = 1, set = 1) uniform accelerationStructureEXT tlas;
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

    vec3 ambient = global.light.ambient_strength * global.light.color * grass_color;
    vec3 diffuse = global.light.diffuse_strength * global.light.color * grass_color * diffuse_factor;
    if (in_shadow()) {
        diffuse = vec3(0);
    }

    vec3 color_at_object = ambient + diffuse;
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
