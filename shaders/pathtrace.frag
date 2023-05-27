#version 460

#extension GL_EXT_ray_query : enable

layout(binding = 2) uniform Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
    float diffuse_strength;
    uint use_ray_tracing;
} light;

layout(binding = 3) uniform accelerationStructureEXT tlas;

layout(location = 0) in vec2 frag_position;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 position = 200 * vec3(0, -frag_position.x, -frag_position.y);

    rayQueryEXT query;
    rayQueryInitializeEXT(query, tlas, gl_RayFlagsTerminateOnFirstHitEXT, 0xff, position, 0.01, vec3(1, 0, 0), 1000.);
    rayQueryProceedEXT(query);
    if (rayQueryGetIntersectionTypeEXT(query, true) != gl_RayQueryCommittedIntersectionNoneEXT) {
        out_color = vec4(1, 1, 1, 1);
    } else {
        out_color = vec4(0, 0, 0, 1);
    }
}
