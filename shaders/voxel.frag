#version 460

#extension GL_EXT_mesh_shader : require
#extension GL_EXT_shader_8bit_storage : require
#extension GL_EXT_shader_16bit_storage : require

#include "types/uniform.glsl"

layout(binding = 0, set = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_position;
layout(location = 1) perprimitiveEXT flat in uint frag_data;
layout(location = 2) perprimitiveEXT flat in vec3 frag_color;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/pbr.glsl"

const vec3 NORMALS[6] = vec3[](
    vec3(1, 0, 0),
    vec3(-1, 0, 0),
    vec3(0, 1, 0),
    vec3(0, -1, 0),
    vec3(0, 0, 1),
    vec3(0, 0, -1)
);

void main() {
    vec3 normal = NORMALS[uint(frag_data) & 7u];
    VoxelMaterial material = global.materials[uint(frag_data) >> 3];
    vec3 reflected_color = pbr(normal, material.albedo, material.metallic, material.roughness);
    vec3 color_at_object = reflected_color + material.emit;
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
