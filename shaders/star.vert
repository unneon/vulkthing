#version 460

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/star.glsl"
#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;
layout(binding = 1, std140) readonly buffer Stars {
    Star stars[];
};

layout(location = 0) in vec3 in_position;

layout(location = 0) out vec3 frag_position;

void main() {
    mat4 star_model = stars[gl_InstanceIndex].model;
    vec4 world_space = star_model * vec4(in_position, 1);
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    frag_position = world_space.xyz;
}
