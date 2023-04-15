#version 450

layout(binding = 0) uniform sampler2D render;
layout(binding = 1) uniform Filters {
    float exposure;
} filters;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 raw = textureLod(render, gl_FragCoord.xy, 0).rgb;
    vec3 after_exposure = raw * filters.exposure;
    // TODO: White balancing.
    // TODO: Contrast.
    // TODO: Brightness.
    // TODO: Color filtering.
    // TODO: Saturation.
    // TODO: Tone mapping.
    // TODO: Gamma?
    vec3 final = after_exposure;
    out_color = vec4(final, 1);
}
