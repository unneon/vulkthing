#version 460

#extension GL_KHR_shader_subgroup_quad : enable

#include "types/uniform.glsl"

layout(constant_id = 0) const int msaa_samples = 0;

layout(binding = 0, input_attachment_index = 0) uniform subpassInputMS render;

layout(binding = 1, rgba16) uniform writeonly image2D bloom;

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

void main() {
    vec3 pixel_bloom = vec3(0);
    for (int i = 0; i < msaa_samples; ++i) {
        vec3 sample_color = subpassLoad(render, i).rgb;
        float sample_greyscale = dot(sample_color, vec3(0.299, 0.587, 0.114));
        if (sample_greyscale > global.gaussian.threshold) {
            pixel_bloom += sample_color;
        }
    }
    pixel_bloom /= msaa_samples;

    vec3 quad_bloom = pixel_bloom;
    quad_bloom += subgroupQuadSwapHorizontal(quad_bloom);
    quad_bloom += subgroupQuadSwapVertical(quad_bloom);
    quad_bloom /= 4;

    if (gl_SubgroupInvocationID % 4 == 0) {
        imageStore(bloom, ivec2(gl_FragCoord.xy / 2), vec4(pixel_bloom, 1));
    }
}
