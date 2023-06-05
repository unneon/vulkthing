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

bool in_shadow() {
    if (!settings.ray_traced_shadows) {
        return false;
    }

    float light_distance = length(light.position - frag_position);
    vec3 light_dir = normalize(light.position - frag_position);

    rayQueryEXT query;
    rayQueryInitializeEXT(query, tlas, 0, 0xff, frag_position, 0.01, light_dir, light_distance);
    while (rayQueryProceedEXT(query)) {}
    if (rayQueryGetIntersectionTypeEXT(query, true) != gl_RayQueryCommittedIntersectionNoneEXT) {
        float distance = rayQueryGetIntersectionTEXT(query, true);
        return distance < light_distance;
    }
    return false;
}

void main() {
    vec3 object_color = material.diffuse;
    vec3 light_dir = normalize(light.position - frag_position);
    float diffuse_factor = max(dot(light_dir, frag_normal), 0);

    vec3 ambient = light.ambient_strength * light.color * object_color;
    vec3 diffuse = light.diffuse_strength * light.color * object_color * diffuse_factor;
    vec3 emit = material.emit;
    if (in_shadow()) {
        diffuse = vec3(0);
    }

    vec3 result = ambient + diffuse + emit;
    out_color = vec4(result, 1);
}
