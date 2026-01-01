#extension GL_GOOGLE_include_directive : require
#include "../bindings.glsl"

layout(location = 0) out PerVertex {
    vec3 direction;
} per_vertex;

const vec2 POSITIONS[6] = { { 1, 1 }, { 1, -1 }, { -1, -1 }, { 1, 1 }, { -1, -1 }, { -1, 1 } };

void main() {
    uint id = gl_VertexIndex;
    vec4 normalized_clip_space = vec4(POSITIONS[id], 1, 1);
    vec3 direction = normalize((global.camera.inverse_view_matrix * global.camera.inverse_projection_matrix * normalized_clip_space).xyz);

    gl_Position = normalized_clip_space;
    per_vertex.direction = direction;
}
