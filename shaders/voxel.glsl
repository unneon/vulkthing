#include "bindings.glsl"
#include "geometry.glsl"

bool back_cull(VoxelMeshlet meshlet) {
    for (int dx = 0; dx < 2; ++dx) {
        for (int dy = 0; dy < 2; ++dy) {
            for (int dz = 0; dz < 2; ++dz) {
                vec3 world_space = global.voxels.chunk_size * vec3(meshlet.chunk) + meshlet.bound_base + meshlet.bound_size * vec3(dx, dy, dz);
                if (dot(world_space - global.camera.position, global.camera.direction) > 0) {
                    return false;
                }
            }
        }
    }
    return true;
}

bool frustum_cull(VoxelMeshlet meshlet) {
    vec4 screen_aabb;
    if (screen_aabb_projection(voxel_meshlet_world_space_min(meshlet), voxel_meshlet_world_space_max(meshlet), screen_aabb)) {
        // TODO: Report compiler bug to slangc.
        return screen_aabb.x > 1 || screen_aabb.y > 1 || (screen_aabb.z) < -1 || (screen_aabb.w) < -1;
    }
    return false;
}

bool is_inside_root_svo(uvec3 voxel) {
    return voxel.x >= 0 && voxel.x < global.voxels.root_svo_side &&
    voxel.y >= 0 && voxel.y < global.voxels.root_svo_side &&
    voxel.z >= 0 && voxel.z < global.voxels.root_svo_side;
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

