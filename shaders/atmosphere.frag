#version 460

#include "types/uniform.glsl"

layout(binding = 0) uniform sampler2D render_sampler;
layout(binding = 1) uniform sampler2D depth_sampler;
layout(binding = 0, set = 1) uniform GLOBAL_UNIFORM_TYPE global;

layout(location = 0) out vec4 out_color;

#include "lighting/atmosphere.glsl"
#include "util/camera.glsl"

void main() {
    float depth = textureLod(depth_sampler, gl_FragCoord.xy, 0).r;
    vec3 position = world_space_from_depth(depth);
    vec3 color_at_object = textureLod(render_sampler, gl_FragCoord.xy, 0).rgb;
    vec3 color_at_camera = compute_atmosphere(color_at_object, position);
    out_color = vec4(color_at_camera, 1);
}
