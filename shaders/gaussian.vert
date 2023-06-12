#version 460

const vec2 POSITIONS[6] = vec2[](
    vec2(1, 1),
    vec2(1, -1),
    vec2(-1, -1),
    vec2(1, 1),
    vec2(-1, -1),
    vec2(-1, 1)
);

void main() {
    gl_Position = vec4(POSITIONS[gl_VertexIndex], 0, 1);
}
