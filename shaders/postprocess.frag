#version 450

layout(binding = 0) uniform sampler2D render;
layout(binding = 1) uniform Filters {
    vec3 color_filter;
    float exposure;
    float temperature;
    float tint;
    float contrast;
    float brightness;
    float saturation;
    uint tonemapper;
    float gamma;
} filters;

layout(location = 0) out vec4 out_color;

const uint TONEMAPPER_RGB_CLAMPING = 0;
const uint TONEMAPPER_REINHARD = 4;
const uint TONEMAPPER_NARKOWICZ_ACES = 8;
const uint TONEMAPPER_HILL_ACES = 9;

// https://docs.unity3d.com/Packages/com.unity.shadergraph@6.9/manual/White-Balance-Node.html
vec3 apply_white_balance(vec3 color, float temperature, float tint) {
    float t1 = temperature * 10 / 6;
    float t2 = tint * 10 / 6;

    float x = 0.31271 - t1 * (t1 < 0 ? 0.1 : 0.05);
    float standard_illuminant_y = 2.87 * x - 3 * x * x - 0.27509507;
    float y = standard_illuminant_y + t2 * 0.05;

    vec3 w1 = vec3(0.949237, 1.03542, 1.08728);

    float Y = 1;
    float X = Y * x / y;
    float Z = Y * (1 - x - y) / y;
    float L = 0.7328 * X + 0.4296 * Y - 0.1624 * Z;
    float M = -0.7036 * X + 1.6975 * Y + 0.0061 * Z;
    float S = 0.0030 * X + 0.0136 * Y + 0.9834 * Z;
    vec3 w2 = vec3(L, M, S);

    vec3 balance = vec3(w1.x / w2.x, w1.y / w2.y, w1.z / w2.z);

    mat3 LIN_2_LMS_MAT;
    LIN_2_LMS_MAT[0] = vec3(3.90405e-1, 7.08416e-2, 2.31082e-2);
    LIN_2_LMS_MAT[1] = vec3(5.49941e-1, 9.63172e-1, 1.28021e-1);
    LIN_2_LMS_MAT[2] = vec3(8.92632e-3, 1.35775e-3, 9.36245e-1);
    mat3 LMS_2_LIN_MAT;
    LMS_2_LIN_MAT[0] = vec3(2.85847e+0, -2.10182e-1, -4.18120e-2);
    LMS_2_LIN_MAT[1] = vec3(-1.62879e+0, 1.15820e+0, -1.18169e-1);
    LMS_2_LIN_MAT[2] = vec3(-2.48910e-2, 3.24281e-4, 1.06867e+0);

    vec3 lms = LIN_2_LMS_MAT * color;
    lms *= balance;
    return LMS_2_LIN_MAT * lms;
}

vec3 tonemapper_rgb_clamping(vec3 color) {
    return clamp(color, 0, 1);
}

vec3 tonemapper_reinhard(vec3 color) {
    float old_luminance = dot(color, vec3(0.299, 0.587, 0.114));
    float new_luminance = old_luminance / (1 + old_luminance);
    return clamp(color / old_luminance * new_luminance, 0, 1);
}

vec3 tonemapper_narkowicz_aces(vec3 color) {
    return clamp((color * (2.51 * color + 0.03)) / (color * (2.43 * color + 0.59) + 0.14), 0, 1);
}

const mat3 HILL_ACES_INPUT = mat3(
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
);

const mat3 HILL_ACES_OUTPUT = mat3(
    1.60475, -0.10208, -0.00327,
    -0.53108, 1.10813, -0.07276,
    -0.07367, -0.00605, 1.07602
);

vec3 hill_aces_rrt_and_odt_fit(vec3 v) {
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return a / b;
}

vec3 tonemapper_hill_aces(vec3 color) {
    return clamp(HILL_ACES_OUTPUT * hill_aces_rrt_and_odt_fit(HILL_ACES_INPUT * color), 0, 1);
}

vec3 apply_tone_mapping(vec3 color) {
    if (filters.tonemapper == TONEMAPPER_RGB_CLAMPING) {
        return tonemapper_rgb_clamping(color);
    } else if (filters.tonemapper == TONEMAPPER_REINHARD) {
        return tonemapper_reinhard(color);
    } else if (filters.tonemapper == TONEMAPPER_NARKOWICZ_ACES) {
        return tonemapper_narkowicz_aces(color);
    } else if (filters.tonemapper == TONEMAPPER_HILL_ACES) {
        return tonemapper_hill_aces(color);
    } else {
        return vec3(1, 0, 0);
    }
}

void main() {
    // Assume the colors computed by the lighting shader are in [0, infinity) HDR.
    vec3 color = textureLod(render, gl_FragCoord.xy, 0).rgb;

    // Apply camera exposure. Assumes exposure is non-negative.
    color = color * filters.exposure;

    // Apply white balancing. Formulae are complex enough that something might go below 0.
    color = apply_white_balance(color, filters.temperature, filters.tint);
    color = max(color, 0);

    // Apply contrast and brightness in a single formula. Only clamp after both.
    color = filters.contrast * (color - 0.5) + 0.5 + filters.brightness;
    color = max(color, 0);

    // Apply color filter. Assumes color filter is non-negative.
    color = color * filters.color_filter;

    // Apply saturation. Greyscale is weighted, as human eyes perceive some colors as brighter than others. Result can
    // negative if saturation is outside [0, 1] range, and the shader should be able to handle that?
    float greyscale = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(greyscale), color, filters.saturation);
    color = max(color, 0);

    // Apply tone mapping, bringing the colors from [0, infinity] HDR to [0, 1] SDR.
    color = apply_tone_mapping(color);

    // Apply gamma correction. As the last step, the exponent will get multipled with the exponent from conversion to
    // sRGB color space. Doesn't require clamping, as [0,1] to a real power is still [0,1].
    color = pow(color, vec3(filters.gamma));

    out_color = vec4(color, 1);
}
