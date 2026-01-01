#include "bindings.glsl"

bool screen_aabb_projection(vec3 min_coords, vec3 max_coords, out vec4 screen_aabb) {
    mat4x4 projection_view = global.camera.projection_matrix * global.camera.view_matrix;
    vec4 sx = projection_view * vec4(max_coords.x - min_coords.x, 0, 0, 0);
    vec4 sy = projection_view * vec4(0, max_coords.y - min_coords.y, 0, 0);
    vec4 sz = projection_view * vec4(0, 0, max_coords.z - min_coords.z, 0);
    vec4 p0 = projection_view * vec4(min_coords, 1);
    vec4 p1 = p0 + sz;
    vec4 p2 = p0 + sy;
    vec4 p3 = p2 + sz;
    vec4 p4 = p0 + sx;
    vec4 p5 = p4 + sz;
    vec4 p6 = p4 + sy;
    vec4 p7 = p6 + sz;
    if (min(min(min(p0.w, p1.w), min(p2.w, p3.w)), min(min(p4.w, p5.w), min(p6.w, p7.w))) < global.camera.depth_near) {
        return false;
    }
    screen_aabb.xy = min(
    min(min(p0.xy / p0.w, p1.xy / p1.w), min(p2.xy / p2.w, p3.xy / p3.w)),
    min(min(p4.xy / p4.w, p5.xy / p5.w), min(p6.xy / p6.w, p7.xy / p7.w))
    );
    screen_aabb.zw = max(
    max(max(p0.xy / p0.w, p1.xy / p1.w), max(p2.xy / p2.w, p3.xy / p3.w)),
    max(max(p4.xy / p4.w, p5.xy / p5.w), max(p6.xy / p6.w, p7.xy / p7.w))
    );
    return true;
}
