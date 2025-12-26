import bindings;

public Optional<float4> screen_aabb_projection(float3 min_coords, float3 max_coords) {
    float4x4 projection_view = mul(global.camera.projection_matrix, global.camera.view_matrix);
    float4 sx = mul(projection_view, float4(max_coords.x - min_coords.x, 0, 0, 0));
    float4 sy = mul(projection_view, float4(0, max_coords.y - min_coords.y, 0, 0));
    float4 sz = mul(projection_view, float4(0, 0, max_coords.z - min_coords.z, 0));
    float4 p0 = mul(projection_view, float4(min_coords, 1));
    float4 p1 = p0 + sz;
    float4 p2 = p0 + sy;
    float4 p3 = p2 + sz;
    float4 p4 = p0 + sx;
    float4 p5 = p4 + sz;
    float4 p6 = p4 + sy;
    float4 p7 = p6 + sz;
    if (min(min(min(p0.w, p1.w), min(p2.w, p3.w)), min(min(p4.w, p5.w), min(p6.w, p7.w))) < global.camera.depth_near) {
        return none;
    }
    float4 screen_aabb;
    screen_aabb.xy = min(
        min(min(p0.xy / p0.w, p1.xy / p1.w), min(p2.xy / p2.w, p3.xy / p3.w)),
        min(min(p4.xy / p4.w, p5.xy / p5.w), min(p6.xy / p6.w, p7.xy / p7.w))
    );
    screen_aabb.zw = max(
        max(max(p0.xy / p0.w, p1.xy / p1.w), max(p2.xy / p2.w, p3.xy / p3.w)),
        max(max(p4.xy / p4.w, p5.xy / p5.w), max(p6.xy / p6.w, p7.xy / p7.w))
    );
    return screen_aabb;
}
