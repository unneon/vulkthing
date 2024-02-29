#version 460

#extension GL_EXT_shader_8bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_position;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"

void main() {
    vec3 color_at_object = vec3(10);
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
