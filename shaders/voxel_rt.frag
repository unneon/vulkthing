#version 460

#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

#include "util/camera.glsl"

void main() {
    vec3 camera_position_within_cube = mod(global.camera.position, 1);
    vec3 view_direction = normalize(world_space_from_depth(1));

    vec3 voxel = floor(global.camera.position);
    vec3 step = sign(view_direction);
    vec3 t_max = abs(((step + 1) / 2 - camera_position_within_cube) / view_direction);
    vec3 t_delta = abs(1 / view_direction);

    vec3 color;

    // Amanatides J, Woo A. A fast voxel traversal algorithm for ray tracing. In Eurographics 1987 Aug 24 (Vol. 87, No. 3, pp. 3-10).
    while (true) {
        if (t_max.x <= t_max.y && t_max.x <= t_max.z) {
            voxel.x += step.x;
            if (abs(voxel.x) > 100) {
                discard;
            }
            t_max.x += t_delta.x;
        } else if (t_max.y <= t_max.x && t_max.y <= t_max.z) {
            voxel.y += step.y;
            if (abs(voxel.y) > 100) {
                discard;
            }
            t_max.y += t_delta.y;
        } else if (t_max.z <= t_max.x && t_max.z <= t_max.y) {
            voxel.z += step.z;
            if (abs(voxel.z) > 100) {
                discard;
            }
            t_max.z += t_delta.z;
        } else {
            discard;
        }
        if (abs(voxel.x) >= 50) {
            color = vec3(1, 0, 0);
            break;
        }
        if (voxel.x > 20 && voxel.x < 30 && voxel.y > 20 && voxel.y < 30 && voxel.z > 20 && voxel.z < 30 && mod(voxel.x + voxel.y + voxel.z, 2) < 1) {
            color = vec3(0, 1, 0);
            break;
        }
    }

    out_color = vec4(color, 1);
}
