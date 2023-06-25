#version 450

layout(binding = 0) uniform sampler2D render;

layout(binding = 1) uniform Gaussian {
    ivec2 step;
    float threshold;
    int radius;
    float exponent_coefficient;
} gaussian;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 total = vec3(0);
    ivec2 coord = ivec2(gl_FragCoord.xy) - gaussian.radius * gaussian.step;
    for (int d = -gaussian.radius; d <= gaussian.radius; ++d, coord += gaussian.step) {
        float factor = exp(-gaussian.exponent_coefficient * d * d);
        total += factor * textureLod(render, coord, 0).rgb;
    }
    out_color = vec4(total, 1);
}
