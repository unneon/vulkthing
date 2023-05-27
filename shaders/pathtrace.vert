#version 450

const vec2 POSITIONS[6] = vec2[](
    vec2(1, 1),
    vec2(1, -1),
    vec2(-1, -1),
    vec2(1, 1),
    vec2(-1, -1),
    vec2(-1, 1)
);

layout(location = 0) out vec2 frag_position;

void main() {
    gl_Position = vec4(POSITIONS[gl_VertexIndex], 0, 1);
    frag_position = POSITIONS[gl_VertexIndex];
}
