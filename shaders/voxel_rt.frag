#version 460
#extension GL_EXT_debug_printf : require
#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require
#include "types/uniform.glsl"
#include "types/voxel.glsl"
#include "util/camera.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;
layout(binding = 5) readonly buffer SvoNodes { SvoNode svo_nodes[]; };
layout(location = 0) out vec4 out_color;

bool is_voxel_inside_root_svo(uvec3 voxel) {
    return
        voxel.x >= global.voxels.root_svo_base.x &&
        voxel.y >= global.voxels.root_svo_base.y &&
        voxel.z >= global.voxels.root_svo_base.z &&
        voxel.x < global.voxels.root_svo_base.x + global.voxels.root_svo_side &&
        voxel.y < global.voxels.root_svo_base.y + global.voxels.root_svo_side &&
        voxel.z < global.voxels.root_svo_base.z + global.voxels.root_svo_side;
}

uint find_svo(uvec3 key) {
    uint svo_index = global.voxels.root_svo_index;
    uint side_length = global.voxels.root_svo_side;
    while (true) {
        SvoNode svo = svo_nodes[svo_index];
        uvec3 child_vec = key / (side_length / 2);
        uint child_index = 4 * child_vec.z + 2 * child_vec.y + child_vec.x;
        uint child = svo.children[child_index];
        bool is_uniform = bitfieldExtract(child, 31, 1) != 0;
        if (is_uniform) {
            uint material = bitfieldExtract(child, 0, 5);
            return material;
        }
        svo_index = child;
        side_length /= 2;
        key = key % side_length;
    }
}

void main() {
    if (uint(gl_FragCoord.x) % 8 >= 4 || uint(gl_FragCoord.y) % 8 >= 4) {
        discard;
    }

    vec3 camera_position_within_cube = mod(global.camera.position, 1);
    vec3 view_direction = normalize(world_space_from_depth(1, global.camera));

    ivec3 voxel = ivec3(floor(global.camera.position));
    ivec3 step = ivec3(sign(view_direction));
    vec3 t_max = abs(((step + 1) / 2 - camera_position_within_cube) / view_direction);
    vec3 t_delta = abs(1 / view_direction);

    if (!is_voxel_inside_root_svo(voxel)) {
        discard;
    }

    // Amanatides J, Woo A. A fast voxel traversal algorithm for ray tracing. In Eurographics 1987 Aug 24 (Vol. 87, No. 3, pp. 3-10).
    uint material_index = find_svo(uvec3(voxel));
    while (material_index == 0) {
        if (t_max.x <= t_max.y && t_max.x <= t_max.z) {
            voxel.x += step.x;
            if (voxel.x < global.voxels.root_svo_base.x || voxel.x >= global.voxels.root_svo_base.x + global.voxels.root_svo_side) {
                discard;
            }
            t_max -= t_max.x;
            t_max.x += t_delta.x;
        } else if (t_max.y <= t_max.z) {
            voxel.y += step.y;
            if (voxel.y < global.voxels.root_svo_base.y || voxel.y >= global.voxels.root_svo_base.y + global.voxels.root_svo_side) {
                discard;
            }
            t_max -= t_max.y;
            t_max.y += t_delta.y;
        } else {
            voxel.z += step.z;
            if (voxel.z < global.voxels.root_svo_base.z || voxel.z >= global.voxels.root_svo_base.z + global.voxels.root_svo_side) {
                discard;
            }
            t_max -= t_max.z;
            t_max.z += t_delta.z;
        }
        material_index = find_svo(uvec3(voxel));
    }

    VoxelMaterial material = global.materials[material_index];
    vec3 color = material.albedo;
    out_color = vec4(color, 1);
}
