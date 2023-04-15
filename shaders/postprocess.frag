#version 450

layout(binding = 0) uniform sampler2D render;
layout(binding = 1) uniform Filters {
    vec3 color_filter;
    float exposure;
    float contrast;
    float brightness;
    float saturation;
    float gamma;
} filters;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 color = textureLod(render, gl_FragCoord.xy, 0).rgb;
    color = clamp(color, 0, 1);

    // Apply camera exposure.
    color = color * filters.exposure;
    color = clamp(color, 0, 1);

    // TODO: Apply white balancing.

    // Apply contrast and brightness in a single formula. Clamping is unnecessary.
    color = filters.contrast * (color - 0.5) + 0.5 + filters.brightness;
    color = clamp(color, 0, 1);

    // Apply color filter.
    color = color * filters.color_filter;
    color = clamp(color, 0, 1);

    // Apply saturation. Greyscale is weighted, as human eyes perceive some colors as brighter than others.
    float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(greyscale), color, filters.saturation);
    color = clamp(color, 0, 1);

    // TODO: Tone mapping.

    // Apply gamma correction. As the last step, the exponent will get multipled with the exponent from conversion to
    // sRGB color space. Doesn't require clamping, as [0,1] to a real power is still [0,1].
    color = pow(color, vec3(filters.gamma));

    out_color = vec4(color, 1);
}
