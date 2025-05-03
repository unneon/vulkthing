module voxel;

import global;

bool is_inside_root_svo(uint3 voxel) {
    return voxel.x >= 0 && voxel.x < global.voxels.root_svo_side &&
           voxel.y >= 0 && voxel.y < global.voxels.root_svo_side &&
           voxel.z >= 0 && voxel.z < global.voxels.root_svo_side;
}

public uint find_svo(uint3 key) {
    uint svo_index = global.voxels.root_svo_index;
    uint side_length = global.voxels.root_svo_side;
    while (true) {
        SvoNode svo = svo_nodes[svo_index];
        uint3 child_vec = key / (side_length / 2);
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

public uint3 ray_trace(int3 ray_position, float3 ray_position_frac, float3 ray_direction) {
    // Amanatides J, Woo A. A fast voxel traversal algorithm for ray tracing. In Eurographics 1987 Aug 24 (Vol. 87, No. 3, pp. 3-10).

    int3 voxel = ray_position;
    int3 step = int3(sign(ray_direction));
    float3 t_max = abs(((step + 1) / 2 - ray_position_frac) / ray_direction);
    float3 t_delta = abs(1 / ray_direction);

    if (!is_inside_root_svo(voxel)) {
        discard;
    }

    uint material_index = find_svo(uint3(voxel));
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
        material_index = find_svo(uint3(voxel));
    }

    // TODO: Return information about sub-voxel position of the hit.
    // The original algorithm avoids FLOPs as much as possible, and from what I tried earlier t_max is not sufficient
    // for recovering sub-voxel info reliably (might be wrong though). This makes sense if voxels are supposed to be
    // your smallest unit of lighting etc., but I want lighting to be aware of voxels (ambient occlusion and such) for
    // stylization purposes, so I'll need to keep track of that. Performance on modern hardware will be much more about
    // memory accesses and divergence anyway.
    return voxel;
}
