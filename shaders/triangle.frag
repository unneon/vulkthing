#version 450

layout(binding = 1) uniform sampler2D tex_sampler;
layout(binding = 2) uniform Material {
    vec3 emit;
} material;
layout(binding = 3) uniform Light {
    vec3 color;
    float ambient_strength;
    vec3 position;
} light;

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 object_color = texture(tex_sampler, frag_tex_coord);
//    vec3 light_dir = normalize(light.position - frag_position);

//    vec3 ambient = light.ambient_strength * light.color * object_color;
//    vec3 diffuse = max(dot(frag_normal, light_dir), 0.) * light.color * object_color;
//    vec3 emit = material.emit;

//    vec3 result = ambient + diffuse + emit;
    float noise = (object_color.r + object_color.g * 0.5 + 0.25 * object_color.b + 0.125 * object_color.a) / 1.875;
    out_color = vec4(0.78 * noise, 0, 0.92 * noise, 1);
}
