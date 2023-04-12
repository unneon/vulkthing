#version 450

layout(binding = 0) uniform sampler2D render;

layout(location = 0) in vec2 frag_position;

layout(location = 0) out vec4 out_color;

vec3 relative(float x, float y) {
    float x_tile = floor(gl_FragCoord.y / 50);
    float y_tile = floor(gl_FragCoord.x / 50);
    float rng = sin(x_tile * 907 + y_tile);
    vec2 center = (frag_position + vec2(1)) / 2;
    vec2 offset = vec2(x * cos(rng) - y * sin(rng), x * sin(rng) + y * cos(rng));
    return texture(render, center + offset * 0.01).rgb;
}

void main() {
    vec3 center = relative(0, 0);
    vec3 sides = relative(-1, 0) + relative(1, 0) + relative(0, -1) + relative(0, 1);
    vec3 corners = relative(-1, -1) + relative(-1, 1) + relative(1, -1) + relative(1, 1);
    vec3 gaussian_approx = 4 * center + 2 * sides + corners;
    out_color = vec4(center * vec3(center.r <= 1, center.g <= 1, center.b <= 1), 1);
}
