#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform sampler2D render;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 color = textureLod(render, gl_FragCoord.xy, 0).rgb;
    float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
    if (greyscale < global.gaussian.threshold) {
        color = vec3(0);
    }
    out_color = vec4(color, 1);
}
