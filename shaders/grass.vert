#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform Transform {
    mat4 model_matrix;
} planet_transform;

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

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
    float blade_height = global.grass.height_average + blade_height_noise * global.grass.height_max_variance;
    float sway_arg = global.grass.sway_frequency * global.grass.time;
    vec3 blade_swayed_up = blade_up + sin(sway_arg) * global.grass.sway_amplitude * global.grass.sway_direction;
    vec3 world_space = (planet_transform.model_matrix * vec4(blade_position, 1)).xyz + naive_x * blade_right * global.grass.width + naive_y * blade_swayed_up * blade_height;
    gl_Position = global.camera.projection_matrix * global.camera.view_matrix * vec4(world_space, 1);
    frag_position = world_space;
    frag_normal = blade_front;
    frag_ground_normal = ground_normal;
    frag_naive_height = naive_y;
}
