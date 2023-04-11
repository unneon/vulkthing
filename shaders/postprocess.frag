#version 450

layout(input_attachment_index = 0, binding = 0) uniform subpassInput t;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(subpassLoad(t).b, 0., 0., 1.);
}
