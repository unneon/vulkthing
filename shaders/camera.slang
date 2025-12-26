module camera;

import bindings;

public float3 world_space_from_depth(float depth, float4 frag_coord) {
    // Algorithms I found didn't work, but there's a simple way to reconstruct the formulae: save the intermediate
    // coordinate spaces to multiple render targets, and match the outputs starting from clip space at each step using
    // an if with gl_FragCoord.x modulo 4 to see if the current formula is correct. You can also render the norm of the
    // difference between vectors, which is more sensitive (make sure to use R32G32B32A32 for MRTs in that case).
    float2 window_space = 2 * frag_coord.xy / global.camera.resolution - 1;
    // We can't reconstruct the w coordinate of the clip space, because vertex post-processing divides the other
    // coordinates by w and doesn't store it (Vulkan specification 27.7 Coordinate Transformations). However, view space
    // always has w equal to 1, so we can make clip space w equal to 1 instead, multiply it by the inverse of the
    // projection matrix to get view space with w not equal to 1, and divide that by our new w to get the original view
    // space coordinates.
    float4 normalized_clip_space = float4(window_space, depth, 1);
    float4 unnormalized_view_space = mul(global.camera.inverse_projection_matrix, normalized_clip_space);
    float4 view_space = float4(unnormalized_view_space.xyz / unnormalized_view_space.w, 1);
    float4 world_space = mul(global.camera.inverse_view_matrix, view_space);
    return world_space.xyz;
}
