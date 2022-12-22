#define_import_path bevy_2d_gi_experiment::gi_types

struct LightSource {
    center:    vec2<f32>,
    intensity: f32,
    color:     vec3<f32>,
    radius:    f32,
    falloff:   vec3<f32>,
}

struct LightSourceBuffer {
    count: u32,
    data:  array<LightSource>,
}

struct LightOccluder {
    center:   vec2<f32>,
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

struct GiState {
    gi_frame_counter:   i32,
    ss_probe_size:      i32,
    ss_atlas_cols:       i32,
    ss_atlas_rows:       i32,
    sdf_max_steps:      i32,
    sdf_jitter_contrib: f32,
    gi_ambient:         vec3<f32>,
}

struct AmbientMask {
    center:   vec2<f32>,
    h_extent: vec2<f32>,
}

struct AmbientMaskBuffer {
    count: u32,
    data:  array<LightOccluder>,
}
