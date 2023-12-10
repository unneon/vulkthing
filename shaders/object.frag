#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

#include "types/uniform.glsl"

layout(binding = 1) uniform MATERIAL_UNIFORM_TYPE material;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;
#ifdef SUPPORTS_RAYTRACING
layout(binding = 1, set = 1) uniform accelerationStructureEXT tlas;
#endif

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/pbr.glsl"
#include "lighting/raytracing.glsl"

void main() {
    vec3 color = pbr(material.albedo, material.metallic, material.roughness, material.ao, material.emit);
    out_color = vec4(color, 1);
}
