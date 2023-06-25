bool in_shadow() {
#ifdef SUPPORTS_RAYTRACING
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
#endif

    return false;
}
