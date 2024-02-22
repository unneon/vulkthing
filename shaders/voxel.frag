#version 460

#extension GL_EXT_mesh_shader : require

#include "types/uniform.glsl"

layout(binding = 0, set = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_position;
layout(location = 1) perprimitiveEXT flat in vec3 frag_color;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/pbr.glsl"

void main() {
    vec3 frag_normal = vec3(0, 0, 1);
    uint frag_material = 3;
    VoxelMaterial material = global.materials[frag_material];
    vec3 reflected_color = pbr(frag_normal, frag_color, material.metallic, material.roughness, 0);
    vec3 color_at_object = reflected_color + material.emit;
    vec3 color_at_camera = compute_atmosphere(color_at_object, frag_position);
    out_color = vec4(color_at_camera, 1);
}
