#version 460

#extension GL_EXT_ray_query : enable

layout(binding = 1) uniform Material {
    vec3 diffuse;
    vec3 emit;
} material;

layout(binding = 2) uniform Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
    float diffuse_strength;
} light;

layout(binding = 3) uniform FragSettings {
    bool ray_traced_shadows;
} settings;

layout(binding = 4) uniform accelerationStructureEXT tlas;

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 object_color = material.diffuse;
    vec3 light_dir = normalize(light.position - frag_position);

    vec3 ambient = light.ambient_strength * light.color * object_color;
    vec3 diffuse = max(dot(frag_normal, light_dir), 0) * light.diffuse_strength * light.color * object_color;
    vec3 emit = material.emit;

    if (settings.ray_traced_shadows) {
        rayQueryEXT query;
        rayQueryInitializeEXT(query, tlas, gl_RayFlagsTerminateOnFirstHitEXT, 0xff, frag_position, 0.01, light_dir, 1000.);
        rayQueryProceedEXT(query);
        if (rayQueryGetIntersectionTypeEXT(query, true) != gl_RayQueryCommittedIntersectionNoneEXT) {
            diffuse *= 0.02;
        }
    }

    vec3 result = ambient + diffuse + emit;
    out_color = vec4(result, 1);
}
