vec2 ray_sphere(vec3 sphere_centre, float sphere_radius, vec3 ray_origin, vec3 ray_direction) {
    vec3 offset = ray_origin - sphere_centre;
    float a = 1;
    float b = 2 * dot(offset, ray_direction);
    float c = dot(offset, offset) - sphere_radius * sphere_radius;
    float d = b * b - 4 * a * c;
    if (d > 0) {
        float s = sqrt(d);
        float distance_to_sphere_near = max(0, (-b - s) / (2 * a));
        float distance_to_sphere_far = (-b + s) / (2 * a);
        if (distance_to_sphere_far >= 0) {
            return vec2(distance_to_sphere_near, distance_to_sphere_far - distance_to_sphere_near);
        }
    }
    return vec2(1. / 0., 0);
}

float density_at_point(vec3 point) {
    float height_above_surface = length(point - global.atmosphere.planet_position) - global.atmosphere.planet_radius;
    float height_01 = height_above_surface / (global.atmosphere.scale * global.atmosphere.planet_radius - global.atmosphere.planet_radius);
    float local_density = exp(-height_01 * global.atmosphere.density_falloff);
    return local_density;
}

// Dividing the segment into n subsegments of equal length and evaluating the function at their midpoints lets us avoid
// precision problems. Might be worth replacing with a more complex numerical integration algorithm or precomputing
// these values later.

float optical_depth(vec3 ray_origin, vec3 ray_direction, float ray_length) {
    float step_length = ray_length / global.atmosphere.optical_depth_point_count;
    vec3 sample_point = ray_origin + ray_direction * step_length / 2;
    float optical_depth = 0;
    for (uint i = 0; i < global.atmosphere.optical_depth_point_count; ++i) {
        float local_density = density_at_point(sample_point);
        optical_depth += local_density * step_length;
        sample_point += ray_direction * step_length;
    }
    return optical_depth;
}

// Computes how much light gets scattered depending on the cosine of the angle. Takes the cosine rather than the angle,
// because that's cheaper to compute given two vectors. Can return results greater than 1 for some reason.
float phase_function(float cos_theta) {
    // The formula is claimed to be "adaptation" of the Henyey-Greenstein function, but it's not clear what was changed
    // and why. Probably a good idea to read the original paper later.
    // https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-16-accurate-atmospheric-scattering
    float g = global.atmosphere.henyey_greenstein_g;
    float c = cos_theta;
    return (3 * (1 - g * g)) / (2 * (2 + g * g))
        * (1 + c * c) / pow(1 + g * g - 2 * g * c, 1.5);
}

vec3 calculate_light(vec3 ray_origin, vec3 ray_direction, float ray_length, vec3 original_color) {
    // This entire function approach with assigning wavelengths to color channels is completely broken, given the output
    // is in sRGB color space. Fixing will come later, as I need to figure out how this should interact with the rest of
    // the rendering pipeline, especially the ACES tone mapping later. There might be resources on this somewhere?
    vec3 scatter_coefficients = global.atmosphere.scattering_strength * vec3(
        pow(400 / global.atmosphere.wavelengths.r, 4),
        pow(400 / global.atmosphere.wavelengths.g, 4),
        pow(400 / global.atmosphere.wavelengths.b, 4)
    );
    float step_length = ray_length / global.atmosphere.scatter_point_count;
    vec3 in_scatter_point = ray_origin + ray_direction * step_length / 2;
    vec3 in_scattered_light = vec3(0);
    for (uint i = 0; i < global.atmosphere.scatter_point_count; ++i) {
        // This is kind of wrong because the sun ray ignores the planet, which results in sunsets being red regardless
        // of the direction you look in (rather than black color when looking away from the sun). Naive approach with
        // ray_sphere results in color banding for some reason? Probably I should be smarter when integrating over
        // in_scatter_point, so that I don't waste precision on scattering points where light is obstructed by the
        // planet.
        // TODO: Account for planet obstructing sun rays.
        vec3 sun_direction = normalize(global.atmosphere.sun_position - in_scatter_point);
        float sun_ray_length = ray_sphere(global.atmosphere.planet_position, global.atmosphere.scale * global.atmosphere.planet_radius, in_scatter_point, sun_direction).y;
        float sun_ray_optical_depth = optical_depth(in_scatter_point, sun_direction, sun_ray_length);
        float view_ray_optical_depth = optical_depth(in_scatter_point, -ray_direction, step_length * i);
        vec3 transmittance = exp(-scatter_coefficients * (sun_ray_optical_depth + view_ray_optical_depth));
        float local_density = density_at_point(in_scatter_point);
        float cos_angle = dot(normalize(sun_direction), normalize(-ray_direction));
        in_scattered_light += local_density * phase_function(cos_angle) * transmittance * scatter_coefficients * step_length;
        in_scatter_point += ray_direction * step_length;
    }
    float original_optical_depth = optical_depth(ray_origin, ray_direction, ray_length);
    vec3 original_transmittance = exp(-scatter_coefficients * original_optical_depth);
    vec3 original_light = original_transmittance * original_color;
    return in_scattered_light + original_light;
}

vec3 compute_atmosphere_impl(vec3 original_color, vec3 ray_direction, float scene_depth) {
    if (!global.atmosphere.enable) {
        return original_color;
    }

    vec3 ray_origin = global.camera.position;
    vec2 hit_info = ray_sphere(global.atmosphere.planet_position, global.atmosphere.scale * global.atmosphere.planet_radius, ray_origin, ray_direction);
    float distance_to_atmosphere = hit_info.x;
    float distance_through_atmosphere = min(hit_info.y, scene_depth - distance_to_atmosphere);

    if (distance_through_atmosphere > 0) {
        vec3 point_in_atmosphere = ray_origin + ray_direction * distance_to_atmosphere;
        return calculate_light(point_in_atmosphere, ray_direction, distance_through_atmosphere, original_color);
    }
    return original_color;
}

vec3 compute_atmosphere(vec3 original_color, vec3 position) {
    if (!global.atmosphere.enable) {
        return original_color;
    }

    vec3 ray_direction = normalize(position - global.camera.position);
    float scene_depth = length(position - global.camera.position);

    return compute_atmosphere_impl(original_color, ray_direction, scene_depth);
}
