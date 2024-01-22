bool in_shadow(vec3 light_position) {
#ifdef SUPPORTS_RAYTRACING
    if (!global.settings.ray_traced_shadows) {
        return false;
    }

    float light_distance = length(light_position - frag_position);
    vec3 light_dir = normalize(light_position - frag_position);

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

const float PHI = 1.61803398874989484820459;
float gold_noise(vec2 xy, float seed){
    return fract(tan(distance(xy*PHI, xy)*seed)*xy.x);
}

bool in_shadow_denoised() {
    float light_offset_x = gold_noise(vec2(gl_FragCoord.x, gl_FragCoord.y), global.light.shadow_sample_seed);
    float light_offset_y = gold_noise(vec2(gl_FragCoord.x, gl_FragCoord.y), global.light.shadow_sample_seed + 1);
    float light_offset_z = gold_noise(vec2(gl_FragCoord.x, gl_FragCoord.y), global.light.shadow_sample_seed + 2);
    vec3 light_offset = vec3(light_offset_x, light_offset_y, light_offset_z);
    return in_shadow(global.light.position + global.light.scale * light_offset);
}
