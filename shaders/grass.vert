#version 450

layout(binding = 0) uniform ModelViewProjection {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

layout(binding = 1) uniform GrassUniform {
    float height_average;
    float height_max_variance;
    float width;
    float time;
    vec3 sway_direction;
    float sway_frequency;
    float sway_amplitude;
} grass;

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 blade_position;
layout(location = 3) in vec3 blade_up;
layout(location = 4) in vec3 blade_right;
layout(location = 5) in vec3 blade_front;
layout(location = 6) in float blade_height_noise;
layout(location = 7) in vec3 ground_normal;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_ground_normal;
layout(location = 3) out float frag_naive_height;

void main() {
    float naive_x = vertex_position.y;
    float naive_y = vertex_position.z;
    float blade_height = grass.height_average + blade_height_noise * grass.height_max_variance;
    float sway_arg = grass.sway_frequency * grass.time;
    vec3 blade_swayed_up = blade_up + sin(sway_arg) * grass.sway_amplitude * grass.sway_direction;
    vec3 position = blade_position + naive_x * blade_right * grass.width + naive_y * blade_swayed_up * blade_height;
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(position, 1);
    frag_position = position;
    frag_normal = blade_front;
    frag_ground_normal = ground_normal;
    frag_naive_height = naive_y;
}
