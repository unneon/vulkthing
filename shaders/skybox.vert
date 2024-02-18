#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec3 frag_direction;

const vec2 POSITIONS[6] = vec2[](
    vec2(1, 1),
    vec2(1, -1),
    vec2(-1, -1),
    vec2(1, 1),
    vec2(-1, -1),
    vec2(-1, 1)
);

void main() {
    vec4 normalized_clip_space = vec4(POSITIONS[gl_VertexIndex], 1, 1);
    gl_Position = normalized_clip_space;
    frag_direction = normalize((global.camera.inverse_view_matrix * global.camera.inverse_projection_matrix * normalized_clip_space).xyz);
}
