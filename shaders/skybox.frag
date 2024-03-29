#version 460

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_direction;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"

void main() {
    out_color = vec4(compute_atmosphere_impl(vec3(0), frag_direction, 1 / 0), 1);
}
