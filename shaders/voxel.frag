#version 460

#include "types/uniform.glsl"

layout(binding = 0, set = 0) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) flat in uint frag_material;

layout(location = 0) out vec4 out_color;

#include "lighting/pbr.glsl"

void main() {
    VoxelMaterial material = global.materials[frag_material];
    vec3 reflected_color = pbr(frag_normal, material.albedo, material.metallic, material.roughness, 0);
    vec3 color = reflected_color + material.emit;
    out_color = vec4(color, 1);
}
