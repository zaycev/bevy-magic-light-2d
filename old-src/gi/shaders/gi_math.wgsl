#define_import_path bevy_magic_light_2d::gi_math
#import bevy_magic_light_2d::gi_types::Quaternion

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

fn distance_squared(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = a - b;
    return dot(c, c);
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(11.9898, 78.233))) * 43758.5453);
}

/// Quaternion Inverse
fn quat_inv(quat: Quaternion) -> Quaternion {
    let q = quat.data;
    // assume it's a unit quaternion, so just Conjugate
    return Quaternion(vec4<f32>( -q.xyz, q.w ));
}

/// Quaternion multiplication
fn quat_dot(quat1: Quaternion, quat2: Quaternion) -> Quaternion {
    let q1 = quat1.data;
    let q2 = quat2.data;
    let scalar = q1.w * q2.w - dot(q1.xyz, q2.xyz);
    let v = cross(q1.xyz, q2.xyz) + q1.w * q2.xyz + q2.w * q1.xyz;
    return Quaternion(vec4<f32>(v, scalar));
}

/// Apply unit quaternion to vector (rotate vector)
fn quat_mul(q: Quaternion, v: vec3<f32>) -> vec3<f32> {
    let r = quat_dot(q, quat_dot(Quaternion(vec4<f32>(v, 0.0)), quat_inv(q)));
    return r.data.xyz;
}