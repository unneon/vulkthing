#version 460

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 in_position;

layout(location = 0) out vec3 frag_position;

void main() {
    vec4 world_space = vec4(50 * in_position + global.light.position, 1);
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    frag_position = world_space.xyz;
}
