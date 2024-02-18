#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 in_position;

layout(location = 0) out vec3 frag_position;

void main() {
    vec4 world_space = vec4(32000 * in_position, 1);
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    frag_position = world_space.xyz;
}
