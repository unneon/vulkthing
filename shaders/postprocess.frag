#version 450

#include "postprocess/white-balance.glsl"
#include "tonemapper/hill-aces.glsl"
#include "tonemapper/narkowicz-aces.glsl"
#include "tonemapper/reinhard.glsl"
#include "tonemapper/rgb-clamping.glsl"
#include "types/uniform.glsl"

layout(constant_id = 0) const int msaa_samples = 0;

layout(binding = 0) uniform sampler2DMS render;
layout(binding = 1) uniform sampler2D bloom;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

vec3 apply_tone_mapping(vec3 color) {
    if (global.postprocessing.tonemapper == TONEMAPPER_RGB_CLAMPING) {
        return rgb_clamping(color);
    } else if (global.postprocessing.tonemapper == TONEMAPPER_REINHARD) {
        return reinhard(color);
    } else if (global.postprocessing.tonemapper == TONEMAPPER_NARKOWICZ_ACES) {
        return narkowicz_aces(color);
    } else if (global.postprocessing.tonemapper == TONEMAPPER_HILL_ACES) {
        return hill_aces(color);
    } else {
        return vec3(1, 0, 0);
    }
}

vec3 postprocess(vec3 color, vec3 bloom_color) {
    // Assume the colors computed by the lighting shader are in [0, infinity) HDR.

    // Apply bloom. It's meant to simulate light overwhelming a camera lens, so I think it should come before exposure?
    color += global.postprocessing.bloom * bloom_color;

    // Apply camera exposure. Assumes exposure is non-negative.
    color = color * global.postprocessing.exposure;

    // Apply white balancing. Formulae are complex enough that something might go below 0.
    color = white_balance(color, global.postprocessing.temperature, global.postprocessing.tint);
    color = max(color, 0);

    // Apply contrast and brightness in a single formula. Only clamp after both.
    color = global.postprocessing.contrast * (color - 0.5) + 0.5 + global.postprocessing.brightness;
    color = max(color, 0);

    // Apply color filter. Assumes color filter is non-negative.
    color = color * global.postprocessing.color_filter;

    // Apply saturation. Greyscale is weighted, as human eyes perceive some colors as brighter than others. Result can
    // negative if saturation is outside [0, 1] range, and the shader should be able to handle that?
    float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(greyscale), color, global.postprocessing.saturation);
    color = max(color, 0);

    // Apply tone mapping, bringing the colors from [0, infinity] HDR to [0, 1] SDR.
    color = apply_tone_mapping(color);

    // Apply gamma correction. As the last step, the exponent will get multipled with the exponent from conversion to
    // sRGB color space. Doesn't require clamping, as [0,1] to a real power is still [0,1].
    color = pow(color, vec3(global.postprocessing.gamma));

    return color;
}

void main() {
    vec3 bloom_color = textureLod(bloom, gl_FragCoord.xy / 2, 0).rgb;
    vec3 total = vec3(0);
    for (int i = 0; i < msaa_samples; ++i) {
        vec3 color = texelFetch(render, ivec2(gl_FragCoord.xy), i).rgb;
        total += postprocess(color, bloom_color);
    }
    out_color = vec4(total / msaa_samples, 1);
}
