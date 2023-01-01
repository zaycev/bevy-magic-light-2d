#define_import_path bevy_magic_light_2d::gi_types

struct LightSource {
    center:    vec2<f32>,
    intensity: f32,
    color:     vec3<f32>,
    falloff:   vec3<f32>,
}

struct LightSourceBuffer {
    count: u32,
    data:  array<LightSource>,
}

struct Quaternion {
    data: vec4<f32>,
}

struct LightOccluder {
    center: vec2<f32>,
    rotation: Quaternion,
    h_extent: vec2<f32>,
}

struct LightOccluderBuffer {
    count: u32,
    data:  array<LightOccluder>,
}

struct ProbeData {
    pose: vec2<f32>,
}

struct ProbeDataBuffer {
    count: u32,
    data:  array<ProbeData>,
}

struct LightPassParams {
    frame_counter:          i32,
    probe_size:             i32,
    probe_atlas_cols:       i32,
    probe_atlas_rows:       i32,
    skylight_color:         vec3<f32>,

    reservoir_size:         u32,
    smooth_kernel_size_h:   u32,
    smooth_kernel_size_w:   u32,
    direct_light_contrib:   f32,
    indirect_light_contrib: f32,
}

struct SkylightMask {
    center:   vec2<f32>,
    h_extent: vec2<f32>,
}

struct SkylightMaskBuffer {
    count: u32,
    data:  array<SkylightMask>,
}

/// Quaternion Inverse
fn quatInv(q: Quaternion) -> Quaternion {
    let q = q.data;
    // assume it's a unit quaternion, so just Conjugate
    return Quaternion(vec4<f32>( -q.xyz, q.w ));
}
/// Quaternion multiplication
fn quatDot(q1: Quaternion, q2: Quaternion) -> Quaternion {
    let q1 = q1.data;
    let q2 = q2.data;
    let scalar = q1.w * q2.w - dot(q1.xyz, q2.xyz);
    let v = cross(q1.xyz, q2.xyz) + q1.w * q2.xyz + q2.w * q1.xyz;
    return Quaternion(vec4<f32>(v, scalar));
}
/// Apply unit quaternion to vector (rotate vector)
fn quatMul(q: Quaternion, v: vec3<f32>) -> vec3<f32> {
    let r = quatDot(q, quatDot(Quaternion(vec4<f32>(v, 0.0)), quatInv(q)));
    return r.data.xyz;
}