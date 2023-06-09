#version 450

layout(binding = 0) uniform ModelViewProjection {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

layout(location = 0) in vec3 vertex_position;

layout(location = 0) out vec3 frag_position;

void main() {
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(vertex_position, 1);
    frag_position = (mvp.model * vec4(vertex_position, 1)).xyz;
}
