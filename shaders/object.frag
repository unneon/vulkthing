#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

#include "types/uniform.glsl"

layout(binding = 1) uniform Material {
    vec3 albedo;
    float metallic;
    vec3 emit;
    float roughness;
    float ao;
} material;

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;
#ifdef SUPPORTS_RAYTRACING
layout(binding = 1, set = 1) uniform accelerationStructureEXT tlas;
#endif

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "lighting/raytracing.glsl"

const float PI = 3.14159265359;

float distribution_ggx(vec3 normal, vec3 halfway, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float ndoth = max(dot(normal, halfway), 0);
    float ndoth2 = ndoth * ndoth;

    float nom = a2;
    float denom = ndoth2 * (a2 - 1) + 1;
    denom = PI * denom * denom;

    return nom / denom;
}

float geometry_schlick_ggx(float ndotv, float roughness) {
    float r = roughness + 1;
    float k = r * r / 8;
    float nom = ndotv;
    float denom = ndotv * (1 - k) + k;
    return nom / denom;
}

float geometry_smith(vec3 normal, vec3 view, vec3 light, float roughness) {
    float ndotv = max(dot(normal, view), 0);
    float ndotl = max(dot(normal, light), 0);
    float ggx2 = geometry_schlick_ggx(ndotv, roughness);
    float ggx1 = geometry_schlick_ggx(ndotl, roughness);
    return ggx1 * ggx2;
}

vec3 fresnel_schlick(float cos_theta, vec3 f0) {
    return f0 + (1 - f0) * pow(clamp(1 - cos_theta, 0, 1), 5);
}

void main() {
    vec3 albedo = material.albedo;
    float metallic = material.metallic;
    float roughness = material.roughness;
    float ao = material.ao;

    vec3 normal = frag_normal;
    vec3 view = normalize(global.camera.position - frag_position);

    vec3 f0 = mix(vec3(0.04), albedo, metallic);

    vec3 light = normalize(global.light.position - frag_position);
    vec3 halfway = normalize(view + light);
    float distance = length(global.light.position - frag_position);
    float attenuation = 1. / (distance * distance);
    vec3 radiance = global.light.color * global.light.intensity * attenuation;

    float ndf = distribution_ggx(normal, halfway, roughness);
    float g = geometry_smith(normal, view, light, roughness);
    vec3 f = fresnel_schlick(max(dot(halfway, view), 0), f0);

    vec3 numerator = ndf * g * f;
    float denominator = 4 * max(dot(normal, view), 0) * max(dot(normal, light), 0) + 0.0001;
    vec3 specular = numerator / denominator;

    vec3 ks = f;
    vec3 kd = (vec3(1) - ks) * (1 - metallic);

    float ndotl = max(dot(normal, light), 0);

    vec3 radiance_out = (kd * albedo / PI + specular) * radiance * ndotl;
    vec3 ambient = vec3(0.03) * albedo * ao;
    vec3 emit = material.emit;

    vec3 color = radiance_out + ambient + emit;

    out_color = vec4(color, 1);
}
