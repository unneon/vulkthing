#version 450

layout(binding = 0) uniform ModelViewProjection {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 blade_position;
layout(location = 3) in vec3 blade_up;
layout(location = 4) in vec3 blade_right;
layout(location = 5) in vec3 blade_front;
layout(location = 6) in float blade_width;
layout(location = 7) in float blade_height;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;

void main() {
    float naive_x = vertex_position.y;
    float naive_y = vertex_position.z;
    vec3 position = blade_position + naive_x * blade_right * blade_width + naive_y * blade_up * blade_height;
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(position, 1);
    frag_position = position;
    frag_normal = blade_front;
}