#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_mesh_shader : require
#include "../bindings.glsl"

layout(location = 0) in float ambient_occlusion;
layout(location = 1) perprimitiveEXT flat in uint triangle_data;

layout(location = 0) out vec4 color;

void main() {
    VoxelTriangleData triangle_data = {uint8_t(triangle_data)};
    VoxelMaterial material = global.materials[voxel_triangle_material_id(triangle_data)];
    vec3 light = (1 - ambient_occlusion) * material.albedo;
    color = vec4(light, 1);
}