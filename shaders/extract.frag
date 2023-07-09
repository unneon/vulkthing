#version 460

#include "types/uniform.glsl"

layout(constant_id = 0) const int msaa_samples = 0;

layout(binding = 0) uniform sampler2DMS render;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 bloom_sum = vec3(0);
    for (int i = 0; i < msaa_samples; ++i) {
        vec3 sample_color = texelFetch(render, ivec2(gl_FragCoord.xy), i).rgb;
        float sample_greyscale = dot(sample_color, vec3(0.299, 0.587, 0.114));
        if (sample_greyscale > global.gaussian.threshold) {
            bloom_sum += sample_color;
        }
    }
    vec3 bloom_color = bloom_sum / msaa_samples;
    out_color = vec4(bloom_color, 1);
}
