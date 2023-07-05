#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

#include "types/uniform.glsl"

layout(binding = 1) uniform Material {
    vec3 diffuse;
    vec3 emit;
} material;

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;
#ifdef SUPPORTS_RAYTRACING
layout(binding = 1, set = 1) uniform accelerationStructureEXT tlas;
#endif

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/raytracing.glsl"

void main() {
    vec3 object_color = material.diffuse;
    vec3 light_dir = normalize(global.light.position - frag_position);
    float diffuse_factor = max(dot(light_dir, frag_normal), 0);

    vec3 ambient = global.light.ambient_strength * global.light.color * object_color;
    vec3 diffuse = global.light.diffuse_strength * global.light.color * object_color * diffuse_factor;
    vec3 emit = material.emit;
    if (in_shadow()) {
        diffuse = vec3(0);
    }

    vec3 color_at_object = ambient + diffuse + emit;
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
