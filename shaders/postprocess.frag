#version 450

layout(binding = 0) uniform sampler2D render;

layout(location = 0) out vec4 out_color;

vec3 relative(float x, float y) {
    vec2 tile = floor(gl_FragCoord.xy / 50);
    float rng = sin(tile.x * 907 + tile.y);
    vec2 offset = vec2(x * cos(rng) - y * sin(rng), x * sin(rng) + y * cos(rng));
    return textureLod(render, gl_FragCoord.xy + 16 * offset, 0).rgb;
}

void main() {
    vec3 center = relative(0, 0);
    vec3 sides = relative(-1, 0) + relative(1, 0) + relative(0, -1) + relative(0, 1);
    vec3 corners = relative(-1, -1) + relative(-1, 1) + relative(1, -1) + relative(1, 1);
    vec3 gaussian_approx = 4 * center + 2 * sides + corners;
    out_color = vec4(gaussian_approx, 1);
}
