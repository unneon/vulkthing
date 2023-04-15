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
    vec3 raw = textureLod(render, gl_FragCoord.xy, 0).rgb;
    vec3 after_exposure = clamp(raw * filters.exposure, 0, 1);
    // TODO: White balancing.
    vec3 after_contrast = filters.contrast * (after_exposure - 0.5) + 0.5;
    vec3 after_brightness = clamp(after_contrast + filters.brightness, 0, 1);
    vec3 after_color_filter = clamp(after_brightness * filters.color_filter, 0, 1);
    vec3 after_saturation = clamp(mix(vec3(dot(after_color_filter, vec3(0.299, 0.587, 0.114))), after_color_filter, filters.saturation), 0, 1);
    // TODO: Tone mapping.
    vec3 after_gamma = pow(after_saturation, vec3(filters.gamma));
    vec3 final = after_gamma;
    out_color = vec4(final, 1);
}
