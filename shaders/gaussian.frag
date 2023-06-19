#version 450

layout(constant_id = 0) const int msaa_samples = 2;

layout(binding = 0) uniform sampler2DMS render;

layout(binding = 1) uniform Gaussian {
    float threshold;
    int radius;
    float exponent_coefficient;
} gaussian;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 total = vec3(0);
    for (int dx = -gaussian.radius; dx <= gaussian.radius; ++dx) {
        for (int dy = -gaussian.radius; dy <= gaussian.radius; ++dy) {
            float factor = exp(-gaussian.exponent_coefficient * (dx * dx + dy * dy));
            ivec2 coord = ivec2(gl_FragCoord.xy) + ivec2(dx, dy);
            for (int i = 0; i < msaa_samples; ++i) {
                vec3 color = texelFetch(render, coord, i).rgb;
                float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
                if (greyscale >= gaussian.threshold) {
                    total += factor * color;
                }
            }
        }
    }
    out_color = vec4(total / msaa_samples, 1);
}
