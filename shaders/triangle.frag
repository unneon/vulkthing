#version 450

layout(binding = 1) uniform sampler2D tex_sampler;

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 object_color = texture(tex_sampler, frag_tex_coord).xyz;
    vec3 light_color = vec3(1.0, 1.0, 1.0);
    vec3 light_pos = vec3(-4.0, 4.0, 4.0);
    vec3 light_dir = normalize(light_pos - frag_position);
    float ambient_strength = 0.1;

    vec3 ambient = ambient_strength * light_color;
    vec3 diffuse = max(dot(frag_normal, light_dir), 0.0) * light_color;

    vec3 result = (ambient + diffuse) * object_color;
    out_color = vec4(result, 1.0);
}
