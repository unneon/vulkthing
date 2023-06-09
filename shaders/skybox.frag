#version 460

layout(location = 0) in vec3 frag_position;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_position;

void main() {
    out_color = vec4(0, 0, 1, 1);
    out_position = vec4(frag_position * 1024, 1);
}
