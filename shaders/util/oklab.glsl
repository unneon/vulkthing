// Oklab perceptual color space utilities. See https://bottosson.github.io/posts/oklab/ for explanation and reference
// implementations.

// Transforms a color from Oklab polar LCh-coordinates to Oklab Lab-coordinates.
vec3 oklab_from_oklch(vec3 oklch) {
    float lightness = oklch.x;
    float chroma = oklch.y;
    float hue = oklch.z;
    float a = chroma * cos(hue);
    float b = chroma * sin(hue);
    return vec3(lightness, a, b);
}

vec3 srgb_from_oklab(vec3 oklab) {
    float lightness = oklab.x;
    float a = oklab.y;
    float b = oklab.z;
    float l_ = lightness + 0.3963377774f * a + 0.2158037573f * b;
    float m_ = lightness - 0.1055613458f * a - 0.0638541728f * b;
    float s_ = lightness - 0.0894841775f * a - 1.2914855480f * b;
    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;
    float red = 4.0767416621f * l - 3.3077115913f * m + 0.2309699292f * s;
    float green = -1.2684380046f * l + 2.6097574011f * m - 0.3413193965f * s;
    float blue = -0.0041960863f * l - 0.7034186147f * m + 1.7076147010f * s;
    return vec3(red, green, blue);
}
