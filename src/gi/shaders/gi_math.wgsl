#define_import_path bevy_2d_gi_experiment::gi_math

// [Drobot2014a] Low Level Optimizations for GCN
fn fast_sqrt(x: f32) -> f32 {
    var bits = bitcast<u32>(x);
        bits = bits >> 1u;
        bits = bits + 0x1fbd1df5u;
    return bitcast<f32>(bits);
}

fn fast_distance_2d(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = a - b;
    return fast_sqrt(d.x * d.x + d.y * d.y);
}

fn fast_length_2d(a: vec2<f32>) -> f32 {
    return fast_sqrt(a.x * a.x + a.y * a.y);
}

fn fast_normalize_2d(a: vec2<f32>) -> vec2<f32> {
    return a / fast_length_2d(a);
}

fn fast_distance_3d(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let d = a - b;
    return fast_sqrt(d.x * d.x + d.y * d.y + d.z * d.z);
}

fn fast_length_3d(a: vec3<f32>) -> f32 {
    return fast_sqrt(a.x * a.x + a.y * a.y + a.z * a.z);
}
