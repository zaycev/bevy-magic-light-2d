#import bevy_magic_light_2d::gi_types::{LightOccluderBuffer, LightPassParams, ProbeDataBuffer}
#import bevy_magic_light_2d::gi_math
#import bevy_magic_light_2d::gi_camera::{CameraParams, screen_to_world, world_to_ndc, ndc_to_screen}
#import bevy_magic_light_2d::gi_halton
#import bevy_magic_light_2d::gi_attenuation

@group(0) @binding(0) var<uniform> camera_params:     CameraParams;
@group(0) @binding(1) var<uniform> cfg:               LightPassParams;
@group(0) @binding(2) var<storage> probes:            ProbeDataBuffer;
@group(0) @binding(3) var          sdf_in:            texture_2d<f32>;
@group(0) @binding(4) var          sdf_in_sampler:    sampler;
@group(0) @binding(5) var          ss_bounce_in:      texture_storage_2d<rgba32float, read>;
@group(0) @binding(6) var          ss_blend_out:      texture_storage_2d<rgba32float, write>;

struct ProbeVal {
    val:       vec3<f32>,
    pose:      vec2<f32>,
}

fn read_probe(
    probe_tile_origin: vec2<i32>,
    probe_tile_pose:   vec2<i32>,
    probe_offset:      vec2<i32>,
    motion_offset:     vec2<f32>,
    tile_size:         vec2<i32>,
    probe_size_f32:    f32) -> ProbeVal {

    let clamped_offset = clamp(probe_tile_pose + probe_offset, vec2<i32>(0), tile_size - vec2<i32>(1));

    // Get position
    let probe_screen_pose = clamped_offset * cfg.probe_size;
    let probe_atlas_pose  = probe_tile_origin + clamped_offset;

    //
    let data        = textureLoad(ss_bounce_in, probe_atlas_pose);
    var val         = data.xyz;

    let halton_offset  = unpack2x16float(bitcast<u32>(data.w)) * probe_size_f32 * 1.0;
    let probe_pose     = screen_to_world(
        probe_screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    ) + halton_offset - motion_offset;

    return ProbeVal(
        val,
        probe_pose,
    );
}

struct SampleResult {
    val:    vec3<f32>,
    weight: f32,
}

fn get_probe_tile_origin(
    probe_id:       i32,
    rows:           i32,
    cols:           i32,
    probe_size:     i32) -> vec2<i32> {

    return vec2<i32>(
        cols,
        rows,
    ) * vec2<i32>(probe_id % probe_size, probe_id / probe_size);
}

fn gauss(x: f32) -> f32 {
    let a = 4.0;
    let b = 0.2;
    let c = 0.05;

    let d = 1.0 / (2.0 * c * c);

    return a * exp(- (x - b) * (x - b) / d);
}

fn estimate_probes_at(
    sample_pose:         vec2<f32>,
    screen_pose:         vec2<i32>,
    probe_id:            i32,
    probe_camera_motion: vec2<f32>,
    tile_size:           vec2<i32>,
    probe_size_f32:      f32) -> SampleResult {

    // Reproject sample world pose to previous frame world pose.
    let reproj_sample_pose     = sample_pose + probe_camera_motion;
    let reproj_ndc             = world_to_ndc(reproj_sample_pose, camera_params.view_proj);

    // Probe pose in the screen.
    let reproj_screen_pose     = ndc_to_screen(reproj_ndc.xy, camera_params.screen_size);

    // Probe pose in tile.
    let reproj_tile_probe_pose = reproj_screen_pose / cfg.probe_size;

    // Get origin position of the probe tile in the atlas.
    let curr_probe_origin      = get_probe_tile_origin(
        probe_id,
        cfg.probe_atlas_rows,
        cfg.probe_atlas_cols,
        cfg.probe_size,
    );

    let base_offset = vec2<i32>(0, 0);
    let base_probe  = read_probe(
        curr_probe_origin,
        reproj_tile_probe_pose,
        base_offset,
        probe_camera_motion,
        tile_size,
        probe_size_f32);

    // Discard if offscreen.
    let base_ndc = world_to_ndc(base_probe.pose, camera_params.view_proj);
    if any(base_ndc <= vec2<f32>(-1.0)) || any(base_ndc >= vec2<f32>(1.0)) {
        return SampleResult(vec3<f32>(0.0), 0.0);
    }

    // Compute bilateral filter with gauss function
    let d = distance(base_probe.pose, sample_pose);
    let g = gauss(d);

    var total_q = base_probe.val * g;
    var total_w = g;

    return SampleResult(
        clamp(total_q, vec3<f32>(0.0), vec3<f32>(1e+4)),
        clamp(total_w, 0.0, 1e+4),
    );
}


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let screen_pose  = vec2<i32>(invocation_id.xy) * cfg.probe_size + cfg.probe_size / 2;
    let sample_pose  = screen_to_world(
        screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    let reservoir_size     = i32(cfg.reservoir_size);
    let curr_probe_id      = cfg.frame_counter % reservoir_size;

    let camera_buffer_size = cfg.probe_size * cfg.probe_size;
    let camera_buffer_id   = cfg.frame_counter;
    let curr_camera_pose   = probes.data[camera_buffer_id].pose;
    let probe_size_f32     = f32(cfg.probe_size);

    let tile_size          = vec2<i32>(camera_params.screen_size / (f32(cfg.probe_size) - 0.001));
    let min_irradiance     = vec3<f32>(0.0);
    let max_irradiance     = vec3<f32>(1e+4);
    var total_irradiance   = min_irradiance;
    var total_weight       = 0.0;

    // Sample radiance from previous frames.
    for (var i = 0; i < reservoir_size; i++) {

        // Get index of probe tile of previous frame.
        var probe_id = curr_probe_id - i;
        if (probe_id < 0) {
            probe_id = reservoir_size + probe_id;
        }

        // Get index of camera of previous frame.
        var probe_camera_buffer_id  = camera_buffer_id - i;
        if (probe_camera_buffer_id < 0) {
            probe_camera_buffer_id = camera_buffer_size + probe_camera_buffer_id;
        }

        // Compute position change.
        let probe_camera_pose   = probes.data[probe_camera_buffer_id].pose;
        let probe_camera_motion = curr_camera_pose - probe_camera_pose;

        // Get sample probe value.
        let r = estimate_probes_at(
            sample_pose,
            screen_pose,
            probe_id,
            probe_camera_motion,
            tile_size,
            probe_size_f32,
        );


        // If probe is active, accumulate irradiance and weight.
        if r.weight > 0.0 {
            total_irradiance += clamp(r.val, min_irradiance, max_irradiance);
            total_weight     += r.weight;
        }
    }


    // Normalize and clamp.
    total_irradiance = total_irradiance / total_weight;
    total_irradiance = clamp(total_irradiance, min_irradiance, max_irradiance);

    var l = vec3<f32>(0.001 + dot(total_irradiance, vec3<f32>(1.0/3.0)));
        l = clamp(vec3<f32>(1.0) - l, vec3<f32>(0.0), vec3<f32>(1.0)) * .15;

    // total_irradiance = log(vec3<f32>(1.0) + total_irradiance + total_irradiance * l);
    total_irradiance = total_irradiance + total_irradiance * l;

    textureStore(ss_blend_out, vec2<i32>(invocation_id.xy), vec4<f32>(total_irradiance, total_weight));
}