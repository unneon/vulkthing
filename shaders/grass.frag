#version 460

#extension GL_EXT_ray_query : enable

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
layout(location = 2) in vec3 frag_ground_normal;
layout(location = 3) in float frag_naive_height;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 object_color = vec3(0.2, 0.8, 0.03);
    float light_distance = length(light.position - frag_position);
    vec3 light_dir = normalize(light.position - frag_position);
    vec3 normal = gl_FrontFacing ? frag_normal : -frag_normal;
    float light_dot = dot(normal, light_dir);

    float light_facing_factor = mix(0.6, 1, light_dot / 2 + 0.5);
    float height_factor = mix(0.5, 1, frag_naive_height);

    vec3 ambient = light.ambient_strength * light.color * object_color;
    vec3 diffuse = light.diffuse_strength * light.color * object_color * light_facing_factor * height_factor;

    if (settings.ray_traced_shadows) {
        rayQueryEXT query;
        rayQueryInitializeEXT(query, tlas, 0, 0xff, frag_position, 0.01, light_dir, light_distance);
        while (rayQueryProceedEXT(query)) {}
        if (rayQueryGetIntersectionTypeEXT(query, true) != gl_RayQueryCommittedIntersectionNoneEXT) {
            float distance = rayQueryGetIntersectionTEXT(query, true);
            if (distance < light_distance) {
                diffuse *= 0.02;
            }
        }
    }

    vec3 result = ambient + diffuse;
    out_color = vec4(result, 1);
}
