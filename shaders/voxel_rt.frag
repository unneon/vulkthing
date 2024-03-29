#version 460
#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require
#include "types/uniform.glsl"
#include "types/voxel.glsl"
#include "util/camera.glsl"

layout(binding = 0) uniform GLOBAL_UNIFORM_TYPE global;
layout(binding = 5) readonly buffer SvoNodes { SvoNode svo_nodes[]; };
layout(location = 0) out vec4 out_color;

struct SvoSearchResult {
    uint index;
    uint child;
    uint side_length;
};

SvoSearchResult find_svo(uint side_length, uvec3 key) {
    uint svo_index = 0;
    while (true) {
        SvoNode svo = svo_nodes[svo_index];
        uvec3 child_vec = key / (side_length / 2);
        uint child_index = 4 * child_vec.z + 2 * child_vec.y + child_vec.x;
        uint child = svo.children[child_index];
        bool is_uniform = bitfieldExtract(child, 31, 1) != 0;
        if (is_uniform) {
            SvoSearchResult result;
            result.index = svo_index;
            result.child = child_index;
            result.side_length = side_length / 2;
            return result;
        }
        svo_index = child;
        side_length /= 2;
        key = key % side_length;
    }
}

void svo_step_up(inout uint svo_index, inout uint svo_child, inout vec3 t_max, inout vec3 t_delta, ivec3 voxel, inout uint side_length, ivec3 step) {
    if (svo_index == 0) {
        discard;
    }
    uint new_index = svo_nodes[svo_index].parent;
    uint new_child = 0;
    if (voxel.x / side_length % 2 == 0) {
        if (step.x > 0) {
            t_max.x += t_delta.x;
        }
    } else {
        if (step.x < 0) {
            t_max.x += t_delta.x;
        }
        new_child += 1;
    }
    if (voxel.y / side_length % 2 == 0) {
        if (step.y > 0) {
            t_max.y += t_delta.y;
        }
    } else {
        if (step.y < 0) {
            t_max.y += t_delta.y;
        }
        new_child += 2;
    }
    if (voxel.z / side_length % 2 == 0) {
        if (step.z > 0) {
            t_max.z += t_delta.z;
        }
    } else {
        if (step.z < 0) {
            t_max.z += t_delta.z;
        }
        new_child += 4;
    }
    svo_index = new_index;
    svo_child = new_child;
    t_delta *= 2;
    side_length *= 2;
}

void svo_step_down(inout uint svo_index, inout uint svo_child, inout vec3 t_max, inout vec3 t_delta, ivec3 voxel, inout uint side_length, ivec3 step) {
    uint new_index = svo_nodes[svo_index].children[svo_child];
    uint new_child = 0;
    t_delta /= 2;
    if (voxel.x % side_length < side_length / 2) {
        if (step.x > 0) {
            t_max.x -= t_delta.x;
        }
    } else {
        if (step.x < 0) {
            t_max.x -= t_delta.x;
        }
        new_child += 1;
    }
    if (voxel.y % side_length < side_length / 2) {
        if (step.y > 0) {
            t_max.y -= t_delta.y;
        }
    } else {
        if (step.y < 0) {
            t_max.y -= t_delta.y;
        }
        new_child += 2;
    }
    if (voxel.z % side_length < side_length / 2) {
        if (step.z > 0) {
            t_max.z -= t_delta.z;
        }
    } else {
        if (step.z < 0) {
            t_max.z -= t_delta.z;
        }
        new_child += 4;
    }
    svo_index = new_index;
    svo_child = new_child;
    side_length /= 2;
}

void main() {
    vec3 camera_position_within_cube = mod(global.camera.position, 1);
    vec3 view_direction = normalize(world_space_from_depth(1, global.camera));

    ivec3 voxel = ivec3(floor(global.camera.position));
    ivec3 step = ivec3(sign(view_direction));
    vec3 t_max = abs(((step + 1) / 2 - camera_position_within_cube) / view_direction);
    vec3 t_delta = abs(1 / view_direction);

    // Amanatides J, Woo A. A fast voxel traversal algorithm for ray tracing. In Eurographics 1987 Aug 24 (Vol. 87, No. 3, pp. 3-10).
    while (!(voxel.x >= 0 && voxel.y >= 0 && voxel.z >= 0 && max(voxel.x, max(voxel.y, voxel.z)) < 64)) {
        if (t_max.x <= t_max.y && t_max.x <= t_max.z) {
            voxel.x += step.x;
            if (abs(voxel.x) >= 256) {
                discard;
            }
            t_max.x += t_delta.x;
        } else if (t_max.y <= t_max.z) {
            voxel.y += step.y;
            if (abs(voxel.y) >= 256) {
                discard;
            }
            t_max.y += t_delta.y;
        } else {
            voxel.z += step.z;
            if (abs(voxel.z) >= 256) {
                discard;
            }
            t_max.z += t_delta.z;
        }
    }

    SvoSearchResult initial_svo = find_svo(64, uvec3(voxel));

    uint t_side_length = 1;
    while (t_side_length < initial_svo.side_length) {
        if (step.x > 0 && (voxel.x % (2 * t_side_length)) / t_side_length == 0) {
            t_max.x += t_delta.x;
        }
        if (step.x < 0 && (voxel.x % (2 * t_side_length)) / t_side_length == 1) {
            t_max.x += t_delta.x;
        }
        if (step.y > 0 && (voxel.y % (2 * t_side_length)) / t_side_length == 0) {
            t_max.y += t_delta.y;
        }
        if (step.y < 0 && (voxel.y % (2 * t_side_length)) / t_side_length == 1) {
            t_max.y += t_delta.y;
        }
        if (step.z > 0 && (voxel.z % (2 * t_side_length)) / t_side_length == 0) {
            t_max.z += t_delta.z;
        }
        if (step.z < 0 && (voxel.z % (2 * t_side_length)) / t_side_length == 1) {
            t_max.z += t_delta.z;
        }
        t_delta *= 2;
        t_side_length *= 2;
    }

    uint svo_index = initial_svo.index;
    uint svo_child = initial_svo.child;

    uint iterations = 0;
    while (true) {
        while (bitfieldExtract(svo_nodes[svo_index].children[svo_child], 31, 1) == 0) {
            svo_step_down(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
        }

        if (bitfieldExtract(svo_nodes[svo_index].children[svo_child], 0, 5) != 0) {
            break;
        }

        if (t_max.x <= t_max.y && t_max.x <= t_max.z) {
            if (step.x > 0) {
                while (svo_child % 2 >= 1) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child += 1;
                voxel.x = voxel.x - voxel.x % int(t_side_length) + int(t_side_length);
            } else {
                while (svo_child % 2 < 1) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child -= 1;
                voxel.x = voxel.x - voxel.x % int(t_side_length) - 1;
            }
            t_max.x += t_delta.x;
        } else if (t_max.y <= t_max.z) {
            if (step.y > 0) {
                while (svo_child % 4 >= 2) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child += 2;
                voxel.y = voxel.y - voxel.y % int(t_side_length) + int(t_side_length);
            } else {
                while (svo_child % 4 < 2) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child -= 2;
                voxel.y = voxel.y - voxel.y % int(t_side_length) - 1;
            }
            t_max.y += t_delta.y;
        } else {
            if (step.z > 0) {
                while (svo_child % 8 >= 4) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child += 4;
                voxel.z = voxel.z - voxel.z % int(t_side_length) + int(t_side_length);
            } else {
                while (svo_child % 8 < 4) {
                    svo_step_up(svo_index, svo_child, t_max, t_delta, voxel, t_side_length, step);
                }
                svo_child -= 4;
                voxel.z = voxel.z - voxel.z % int(t_side_length) - 1;
            }
            t_max.z += t_delta.z;
        }

        if (++iterations == 1000) {
            out_color = vec4(1, 0, 1, 1);
            return;
        }
    }

    uint material_index = bitfieldExtract(svo_nodes[svo_index].children[svo_child], 0, 5);
    if (material_index == 0) {
        discard;
    }
    VoxelMaterial material = global.materials[material_index];
    vec3 color = material.albedo;
    out_color = vec4(color, 1);
}
