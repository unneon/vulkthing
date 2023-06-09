#version 450

#include "postprocess/white-balance.glsl"
#include "tonemapper/hill-aces.glsl"
#include "tonemapper/narkowicz-aces.glsl"
#include "tonemapper/reinhard.glsl"
#include "tonemapper/rgb-clamping.glsl"

layout(constant_id = 0) const int msaa_samples = 0;

layout(binding = 0) uniform sampler2DMS render;
layout(binding = 1) uniform sampler2DMS position;
layout(binding = 2) uniform Postprocessing {
    vec3 color_filter;
    float exposure;
    float temperature;
    float tint;
    float contrast;
    float brightness;
    float saturation;
    uint tonemapper;
    float gamma;
} postprocessing;
layout(binding = 3) uniform Camera {
    vec3 position;
} camera;

layout(location = 0) out vec4 out_color;

const uint TONEMAPPER_RGB_CLAMPING = 0;
const uint TONEMAPPER_REINHARD = 4;
const uint TONEMAPPER_NARKOWICZ_ACES = 8;
const uint TONEMAPPER_HILL_ACES = 9;

vec2 ray_sphere(vec3 sphere_centre, float sphere_radius, vec3 ray_origin, vec3 ray_direction) {
    vec3 offset = ray_origin - sphere_centre;
    float a = 1;
    float b = 2 * dot(offset, ray_direction);
    float c = dot(offset, offset) - sphere_radius * sphere_radius;
    float d = b * b - 4 * a * c;
    if (d > 0) {
        float s = sqrt(d);
        float distance_to_sphere_near = max(0, -(b + s) / (2 * a));
        float distance_to_sphere_far = (s - b) / (2 * a);
        if (distance_to_sphere_far >= 0) {
            return vec2(distance_to_sphere_near, distance_to_sphere_far - distance_to_sphere_near);
        }
    }
    return vec2(1. / 0., 0);
}

vec3 atmosphere(vec3 original_color, vec3 position) {
    if (position == vec3(0)) {
        return vec3(0);
    }
    vec3 atmosphere_centre = vec3(0);
    float atmosphere_radius = 1200;
    float scene_depth = length(position - camera.position);
    vec3 ray_origin = camera.position;
    vec3 ray_direction = normalize(position - camera.position);
    vec2 hit_info = ray_sphere(atmosphere_centre, atmosphere_radius, ray_origin, ray_direction);
    float distance_to_atmosphere = hit_info.x;
    float distance_through_atmosphere = min(hit_info.y, scene_depth - distance_to_atmosphere);
    float fog_factor = distance_through_atmosphere / (atmosphere_radius * 2);
    return mix(vec3(1), original_color, 1 - fog_factor);
}

vec3 apply_tone_mapping(vec3 color) {
    if (postprocessing.tonemapper == TONEMAPPER_RGB_CLAMPING) {
        return rgb_clamping(color);
    } else if (postprocessing.tonemapper == TONEMAPPER_REINHARD) {
        return reinhard(color);
    } else if (postprocessing.tonemapper == TONEMAPPER_NARKOWICZ_ACES) {
        return narkowicz_aces(color);
    } else if (postprocessing.tonemapper == TONEMAPPER_HILL_ACES) {
        return hill_aces(color);
    } else {
        return vec3(1, 0, 0);
    }
}

vec3 postprocess(vec3 color) {
    // Assume the colors computed by the lighting shader are in [0, infinity) HDR.

    // Apply camera exposure. Assumes exposure is non-negative.
    color = color * postprocessing.exposure;

    // Apply white balancing. Formulae are complex enough that something might go below 0.
    color = white_balance(color, postprocessing.temperature, postprocessing.tint);
    color = max(color, 0);

    // Apply contrast and brightness in a single formula. Only clamp after both.
    color = postprocessing.contrast * (color - 0.5) + 0.5 + postprocessing.brightness;
    color = max(color, 0);

    // Apply color filter. Assumes color filter is non-negative.
    color = color * postprocessing.color_filter;

    // Apply saturation. Greyscale is weighted, as human eyes perceive some colors as brighter than others. Result can
    // negative if saturation is outside [0, 1] range, and the shader should be able to handle that?
    float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(greyscale), color, postprocessing.saturation);
    color = max(color, 0);

    // Apply tone mapping, bringing the colors from [0, infinity] HDR to [0, 1] SDR.
    color = apply_tone_mapping(color);

    // Apply gamma correction. As the last step, the exponent will get multipled with the exponent from conversion to
    // sRGB color space. Doesn't require clamping, as [0,1] to a real power is still [0,1].
    color = pow(color, vec3(postprocessing.gamma));

    return color;
}

void main() {
    vec3 total = vec3(0);
    for (int i = 0; i < msaa_samples; ++i) {
        vec3 color = texelFetch(render, ivec2(gl_FragCoord.xy), i).rgb;
        vec3 position = texelFetch(position, ivec2(gl_FragCoord.xy), i).xyz;
        total += atmosphere(color, position);
    }
    out_color = vec4(total / msaa_samples, 1);
}
