#extension GL_GOOGLE_include_directive : require
#include "../bindings.glsl"

void main() {
    vec3 position = classic_vertices[gl_VertexIndex].position;
    vec4 world_space = vec4(50 * position + global.light.position, 1);
    vec4 clip_space = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    gl_Position = clip_space;
}
