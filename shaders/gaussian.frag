#version 450

#include "types/uniform.glsl"

layout(binding = 0) uniform sampler2D render;

layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 total = vec3(0);
    // TODO: Fix only horizontal blur.
    ivec2 step = ivec2(1, 0);
    ivec2 coord = ivec2(gl_FragCoord.xy) - global.gaussian.radius * step;
    for (int d = -global.gaussian.radius; d <= global.gaussian.radius; ++d, coord += step) {
        float factor = exp(-global.gaussian.exponent_coefficient * d * d);
        total += factor * textureLod(render, coord, 0).rgb;
    }
    out_color = vec4(total, 1);
}
