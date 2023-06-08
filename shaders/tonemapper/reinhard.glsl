vec3 reinhard(vec3 color) {
    float old_luminance = dot(color, vec3(0.299, 0.587, 0.114));
    float new_luminance = old_luminance / (1 + old_luminance);
    return clamp(color / old_luminance * new_luminance, 0, 1);
}
