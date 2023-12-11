#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform Transform {
    mat4 model_matrix;
} transform;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;

void main() {
    vec4 world_space = transform.model_matrix * vec4(in_position, 1);
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    frag_position = world_space.xyz;
    frag_normal = in_normal;
}
