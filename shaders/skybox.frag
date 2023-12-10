#version 460

#include "types/uniform.glsl"

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_position;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_position;

#include "lighting/atmosphere.glsl"

void main() {
    out_color = vec4(compute_atmosphere(vec3(0), frag_position), 1);
    out_position = vec4(frag_position, 1);
}
