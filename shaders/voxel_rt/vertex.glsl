const vec2 POSITIONS[6] = { { 1, 1 }, { 1, -1 }, { -1, -1 }, { 1, 1 }, { -1, -1 }, { -1, 1 } };

void main() {
    gl_Position = vec4(POSITIONS[gl_VertexIndex], 0.9, 1);
}
