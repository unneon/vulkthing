#version 460

#ifdef SUPPORTS_RAYTRACING
    #extension GL_EXT_ray_query : enable
#endif

layout(binding = 2) uniform Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
    float diffuse_strength;
} light;

layout(binding = 3) uniform FragSettings {
    bool ray_traced_shadows;
} settings;

#ifdef SUPPORTS_RAYTRACING
layout(binding = 4) uniform accelerationStructureEXT tlas;
#endif

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec3 frag_ground_normal;
layout(location = 3) in float frag_naive_height;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_position;

#ifdef SUPPORTS_RAYTRACING
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
#endif

void main() {
    float height_factor = mix(0.7, 1, frag_naive_height);
    vec3 grass_color = vec3(0.2, 0.8, 0.03) * height_factor;
    float diffuse_factor = 1;

    vec3 ambient = light.ambient_strength * light.color * grass_color;
    vec3 diffuse = light.diffuse_strength * light.color * grass_color * diffuse_factor;
#ifdef SUPPORTS_RAYTRACING
    if (in_shadow()) {
        diffuse = vec3(0);
    }
#endif

    vec3 result = ambient + diffuse;
    out_color = vec4(result, 1);
    out_position = vec4(frag_position, 1);
}
