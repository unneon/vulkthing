#version 450

layout(binding = 0) uniform ModelViewProjection {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_tex_coord;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_tex_coord;

void main() {
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(in_position, 1);
    frag_position = (mvp.model * vec4(in_position, 1)).xyz;
    frag_normal = in_normal;
    frag_tex_coord = in_tex_coord;
}
