// https://docs.unity3d.com/Packages/com.unity.shadergraph@6.9/manual/White-Balance-Node.html
vec3 white_balance(vec3 color, float temperature, float tint) {
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
