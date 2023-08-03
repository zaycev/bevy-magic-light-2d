#define_import_path bevy_magic_light_2d::gi_attenuation

fn distance_squared_two(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = a - b;
    return dot(c, c);
}

fn light_attenuation_r_two(
    sample_pose: vec2<f32>,
    light_pose:  vec2<f32>,
    a: f32, // 100.-
    b: f32, // 30.0
    c: f32, // 0.1
) -> f32 {
    let d = distance_squared_two(light_pose, sample_pose);
    let att = a / (b + c * d);
    return clamp(att, 0.0, 1000.0);
}

fn light_attenuation_r(
    sample_pose: vec2<f32>,
    light_pose:  vec2<f32>,
    a: f32, // 100.
    b: f32, // 10.0
    c: f32, // 0.0001
) -> f32 {
    let d = distance(light_pose, sample_pose);
    let att       = a / (b + c * d);
    return clamp(att, 0.0, 1000.0);
}

fn light_attenuation_at_dist_r(
    d: f32,
    a: f32, // 100.
    b: f32, // 10.0
    c: f32, // 0.0001
) -> f32 {
    let att       = a / (b + c * d);
    return clamp(att, 0.0, 1000.0);
}