#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform Transform {
    mat4 model_matrix;
} transform;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec4 star_model_c1;
layout(location = 2) in vec4 star_model_c2;
layout(location = 3) in vec4 star_model_c3;
layout(location = 4) in vec4 star_model_c4;
layout(location = 5) in vec3 star_emit;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_emit;

void main() {
    mat4 star_model = mat4(star_model_c1, star_model_c2, star_model_c3, star_model_c4);
    vec4 world_space = star_model * transform.model_matrix * vec4(in_position, 1);
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    frag_position = world_space.xyz;
    frag_emit = star_emit;
}
