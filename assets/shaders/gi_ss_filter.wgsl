#import bevy_2d_gi_experiment::gi_math
#import bevy_2d_gi_experiment::gi_types
#import bevy_2d_gi_experiment::gi_camera
#import bevy_2d_gi_experiment::gi_halton
#import bevy_2d_gi_experiment::gi_attenuation

@group(0) @binding(0) var<uniform> camera_params:     CameraParams;
@group(0) @binding(1) var<uniform> state:             GiState;
@group(0) @binding(2) var<storage> probes:            ProbeDataBuffer;
@group(0) @binding(3) var          sdf_in:            texture_storage_2d<r16float,    read>;
@group(0) @binding(4) var          ss_blend_in:       texture_storage_2d<rgba32float, read>;
@group(0) @binding(5) var          ss_filter_out:     texture_storage_2d<rgba32float, write>;

fn distance_squared(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = a - b;
    return dot(c, c);
}

fn get_sdf_screen(screen_pose: vec2<i32>) -> f32 {
    return textureLoad(sdf_in, screen_pose).r;
}

fn raymarch_occlusion(
    ray_origin:    vec2<f32>,
    light_pose:    vec2<f32>,
) -> f32 {

    let max_steps      = 8;
    let ray_direction  = fast_normalize_2d(light_pose - ray_origin);
    let stop_at        = distance_squared(ray_origin, light_pose);

    var ray_progress   = 0.0;
    for (var i: i32 = 0; i < max_steps; i++) {

        if (ray_progress * ray_progress >= stop_at) {
            return 1.0;
        }

        let h          = ray_origin + ray_progress * ray_direction;
        let scene_dist = get_sdf_screen(world_to_screen(h, camera_params.screen_size, camera_params.view_proj));

        if (scene_dist <= 0.1) {
            return 0.0;
        }

        ray_progress += scene_dist;
    }

    return 0.0;
}

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
    let base_probe_grid_pose   = screen_pose / state.ss_probe_size;
    let base_probe_sample      = textureLoad(ss_blend_in, base_probe_screen_pose).xyz;
    let base_probe_world_pose  = screen_to_world(
        base_probe_screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    let kernel_hl = 1;
    let kernel_hr = 1;

    var total_w = 0.0;
    var total_q = vec3<f32>(0.0);
    var total_samples = 0;

    for (var i = -kernel_hl; i <= kernel_hr; i++) {
        for (var j = -kernel_hl; j <= kernel_hr; j++) {

            let offset = vec2<i32>(i, j);

            let p_grid_pose   = base_probe_grid_pose + offset;
            let p_screen_pose = (base_probe_grid_pose + offset) * state.ss_probe_size;

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
            if raymarch_occlusion(sample_world_pose, p_world_pose) <= 0.0 {
                continue;
            }

            let d = fast_distance_2d(p_world_pose, sample_world_pose);
            let x = fast_distance_3d(p_sample, base_probe_sample);
            let g = gauss(x) * gauss(d);

            total_q += p_sample * g;
            total_w += g;
        }
    }

    var irradiance = vec3<f32>(0.0);
    if (total_w > 0.0) {
        irradiance = total_q / total_w;
        // irradiance = lin_to_srgb(total_q / total_w);
    }

    textureStore(ss_filter_out, screen_pose, vec4<f32>(irradiance.xyz, 1.0));
}
