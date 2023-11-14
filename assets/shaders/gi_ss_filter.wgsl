#import bevy_magic_light_2d::gi_types::{LightPassParams, ProbeDataBuffer}
#import bevy_magic_light_2d::gi_math
#import bevy_magic_light_2d::gi_camera::{CameraParams, screen_to_world, screen_to_ndc, world_to_sdf_uv, bilinear_sample_r}
#import bevy_magic_light_2d::gi_halton
#import bevy_magic_light_2d::gi_attenuation
#import bevy_magic_light_2d::gi_raymarch::raymarch_primary

@group(0) @binding(0) var<uniform> camera_params:     CameraParams;
@group(0) @binding(1) var<uniform> cfg:               LightPassParams;
@group(0) @binding(2) var<storage> probes:            ProbeDataBuffer;
@group(0) @binding(3) var          sdf_in:            texture_2d<f32>;
@group(0) @binding(4) var          sdf_in_sampler:    sampler;
@group(0) @binding(5) var          ss_blend_in:       texture_storage_2d<rgba32float, read>;
@group(0) @binding(6) var          ss_filter_out:     texture_storage_2d<rgba32float, write>;
@group(0) @binding(7) var          ss_pose_out:      texture_storage_2d<rg32float, write>;

fn gauss(x: f32) -> f32 {
    let a = 4.0;
    let b = 0.2;
    let c = 0.05;

    let d = 1.0 / (2.0 * c * c);

    return a * exp(- (x - b) * (x - b) / d);
}


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let screen_pose        = vec2<i32>(invocation_id.xy);
    let sample_world_pose  = screen_to_world(
        screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    let base_probe_screen_pose = screen_pose;
    let base_probe_grid_pose   = screen_pose / cfg.probe_size;
    let base_probe_sample      = textureLoad(ss_blend_in, base_probe_screen_pose).xyz;
    let base_probe_world_pose  = screen_to_world(
        base_probe_screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    let kernel_hl = i32(cfg.smooth_kernel_size_w);
    let kernel_hr = i32(cfg.smooth_kernel_size_h);

    var total_w = 0.0;
    var total_q = vec3<f32>(0.0);
    var total_samples = 0;

    for (var i = -kernel_hl; i <= kernel_hr; i++) {
        for (var j = -kernel_hl; j <= kernel_hr; j++) {

            let offset = vec2<i32>(i, j);

            let p_grid_pose   = base_probe_grid_pose + offset;
            let p_screen_pose = (base_probe_grid_pose + offset) * cfg.probe_size;

            // Discard offscreen;
            let p_ndc = screen_to_ndc(p_screen_pose, camera_params.screen_size, camera_params.screen_size_inv);
            if any(p_ndc < vec2<f32>(-1.0)) || any(p_ndc > vec2<f32>(1.0)) {
                continue;
            }

            let p_world_pose = screen_to_world(
                p_screen_pose,
                camera_params.screen_size,
                camera_params.inverse_view_proj,
                camera_params.screen_size_inv,
            );

            let p_sample = textureLoad(ss_blend_in, p_grid_pose).xyz;

            // Discard occluded probes.
            if raymarch_primary(sample_world_pose, p_world_pose,
                8,
                sdf_in,
                sdf_in_sampler,
                camera_params,
                0.0).success <= 0 {
                continue;
            }

            let d = distance(p_world_pose, sample_world_pose);
            let x = distance(p_sample, base_probe_sample);
            let g = gauss(x) * gauss(d);

            total_q += p_sample * g;
            total_w += g;
        }
    }

    var irradiance = vec3<f32>(0.0);
    if (total_w > 0.0) {
        irradiance = total_q / total_w;
    }

    let sdf_uv = world_to_sdf_uv(sample_world_pose, camera_params.view_proj, camera_params.inv_sdf_scale);

    textureStore(ss_filter_out, screen_pose, vec4<f32>(irradiance.xyz, 1.0));
    textureStore(ss_pose_out, screen_pose, vec4<f32>(sdf_uv, 0.0,  0.0));
}
