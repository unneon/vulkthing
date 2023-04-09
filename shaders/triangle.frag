#version 450

layout(binding = 1) uniform sampler2D tex_sampler;
layout(binding = 2) uniform Lighting {
    vec3 color;
    vec3 pos;
} light;

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 object_color = texture(tex_sampler, frag_tex_coord).xyz;
    vec3 light_dir = normalize(light.pos - frag_position);
    float ambient_strength = 0.04;

    vec3 ambient = ambient_strength * vec3(1.0, 1.0, 1.0);
    vec3 diffuse = max(dot(frag_normal, light_dir), 0.0) * light.color;

    vec3 result = (ambient + diffuse) * object_color;
    out_color = vec4(result, 1.0);
}
