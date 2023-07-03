#version 450

layout(binding = 0) uniform ModelViewProjection {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec4 star_model_c1;
layout(location = 2) in vec4 star_model_c2;
layout(location = 3) in vec4 star_model_c3;
layout(location = 4) in vec4 star_model_c4;
layout(location = 5) in vec3 star_emit;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_emit;

void main() {
    mat4 star_model = mat4(star_model_c1, star_model_c2, star_model_c3, star_model_c4);
    gl_Position = mvp.proj * mvp.view * star_model * mvp.model * vec4(vertex_position, 1);
    frag_position = (star_model * mvp.model * vec4(vertex_position, 1)).xyz;
    frag_emit = star_emit;
}
