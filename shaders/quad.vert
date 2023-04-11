#version 450

vec2 positions[6] = vec2[](
    vec2(1, 1),
    vec2(1, -1),
    vec2(-1, -1),
    vec2(1, 1),
    vec2(-1, -1),
    vec2(-1, 1)
);

layout(location = 0) out vec2 frag_position;

void main() {
    vec2 position = positions[gl_VertexIndex];
    gl_Position = vec4(position, 0, 1);
    frag_position = position;
}
