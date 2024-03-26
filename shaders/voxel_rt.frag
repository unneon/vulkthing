#version 460

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

#include "util/camera.glsl"

void main() {
    out_color = vec4(normalize(world_space_from_depth(1)), 1);
}
