#extension GL_GOOGLE_include_directive : require
#include "../bindings.glsl"

void main() {
    vec3 in_position = classic_vertices[gl_VertexIndex].position;
    mat4x4 star_model = stars[gl_InstanceIndex].model;
    vec4 world_space = star_model * vec4(in_position, 1);
    vec4 clip_space = global.camera.projection_matrix * global.camera.view_matrix * world_space;
    gl_Position = clip_space;
}