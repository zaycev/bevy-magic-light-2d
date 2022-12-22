#import bevy_2d_gi_experiment::gi_math
#import bevy_2d_gi_experiment::gi_types
#import bevy_2d_gi_experiment::gi_camera
#import bevy_2d_gi_experiment::gi_attenuation
#import bevy_2d_gi_experiment::gi_halton

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<uniform> state:                 GiState;
@group(0) @binding(2) var<storage> probes:                ProbeDataBuffer;
@group(0) @binding(3) var<storage> ambient_masks_buffer:  AmbientMaskBuffer;
@group(0) @binding(4) var<storage> lights_source_buffer:  LightSourceBuffer;
@group(0) @binding(5) var          sdf_in:                texture_storage_2d<r16float,    read>;
@group(0) @binding(6) var          ss_probe_out:          texture_storage_2d<rgba16float, write>;


fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(11.9898, 78.233))) * 43758.5453);
}

fn distance_squared(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = a - b;
    return dot(c, c);
}

fn get_sdf_screen(screen_pose: vec2<i32>) -> f32 {
    return textureLoad(sdf_in, screen_pose).r;
}

fn get_sdf_world(world_pose: vec2<f32>) -> f32 {
    let ndc = vec4<f32>(world_pose, 0.0, 1.0) * camera_params.view_proj;
    let screen_pose = ndc_to_screen(ndc.xy, camera_params.screen_size);
    return get_sdf_screen(screen_pose);
}

fn raymarch(
    ray_origin:    vec2<f32>,
    light_pose:    vec2<f32>,
    max_steps:     i32,
) -> f32 {

    let rm_max_steps:      i32 = max_steps;
    let rm_jitter_contrib: f32 = 0.5;
    let rm_sdf_contrib:    f32 = 0.1;

    let ray_direction          = fast_normalize_2d(light_pose - ray_origin);
    let stop_at                = distance_squared(ray_origin, light_pose);

    var ray_progress:   f32    = 0.0;
    var light_contrib:  f32    = 1.0;
    var scene_dist:     f32    = 0.0;

    for (var i: i32 = 0; i < rm_max_steps; i++) {

        if (ray_progress * ray_progress >= stop_at) {
            return light_contrib * rm_sdf_contrib + (1.0 - rm_sdf_contrib);
        }

        let h              = ray_origin + ray_progress * ray_direction;
        let h_ndc          = world_to_ndc(h, camera_params.view_proj);
        let h_screen       = ndc_to_screen(h_ndc, camera_params.screen_size);
        let new_scene_dist = get_sdf_screen(h_screen);

        if any(h_ndc < vec2<f32>(-1.0)) || any(h_ndc > vec2<f32>(1.0)) {
            let dist_to_light    = distance_squared(h, light_pose);
            let dist_to_occluder = scene_dist * scene_dist;
            if dist_to_light > dist_to_occluder && scene_dist < new_scene_dist * 0.5 {
                return 0.0;
            } else {
            return light_contrib * rm_sdf_contrib + (1.0 - rm_sdf_contrib);
            }
        }

        scene_dist = new_scene_dist;
        if (scene_dist <= 0.1) {
            return 0.0;
        }

        light_contrib = min(light_contrib, scene_dist / ray_progress);

        // Jitter step.
        let jitter = radical_inverse_vdc(i);
        ray_progress += scene_dist * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * scene_dist * jitter;
    }

    return 0.0;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let reservoir_size           = 8;
    let probe_size_f32           = f32(state.ss_probe_size);
    let frames_max               = state.ss_probe_size * state.ss_probe_size;
    let frame_index              = state.gi_frame_counter % reservoir_size;
    let halton_jitter            = hammersley2d(frame_index, reservoir_size);
    let probe_tile_origin_screen = tile_xy * state.ss_probe_size;

    // Get current frame.
    let probe_offset_world  = halton_jitter * probe_size_f32;
    let probe_center_world  = screen_to_world(
        probe_tile_origin_screen,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    ) + probe_offset_world;

    let probe_ndc    = world_to_ndc(probe_center_world, camera_params.view_proj);
    let probe_screen = ndc_to_screen(probe_ndc, camera_params.screen_size);
    var is_masked    = 1.0;

    // Check if the probe is masked from skylight.
    for (var i: i32 = 0; i < i32(ambient_masks_buffer.count); i++) {
        let mask = ambient_masks_buffer.data[i];
        if probe_center_world.x > mask.center.x - mask.h_extent.x &&
           probe_center_world.x < mask.center.x + mask.h_extent.x &&
           probe_center_world.y > mask.center.y - mask.h_extent.y &&
           probe_center_world.y < mask.center.y + mask.h_extent.y {
            is_masked = 0.0;
            break;
        }
    }

    var probe_irradiance = vec3<f32>(0.0);
    if get_sdf_screen(probe_screen) > 0.0 {

        let ambient = state.gi_ambient * is_masked;;

        // Compute direct irradiance from lights in the current frame.
        probe_irradiance = vec3<f32>(ambient);
        for (var i: i32 = 0; i < i32(lights_source_buffer.count); i++) {

            let light = lights_source_buffer.data[i];

            let occlusion = raymarch(
                probe_center_world,
                light.center,
                32,
            );

            let att = light_attenuation_r2(
                probe_center_world,
                light.center,
                light.falloff.x,
                light.falloff.y,
                light.falloff.z,
            );

            probe_irradiance += light.color * occlusion * att * light.intensity;
        }

    }

    // Coordinates of the screen-space cache output tile.
    let atlas_row  = frame_index / state.ss_probe_size;
    let atlas_col  = frame_index % state.ss_probe_size;


    let out_atlas_tile_offset = vec2<i32>(
        state.ss_atlas_cols * atlas_col,
        state.ss_atlas_rows * atlas_row,
    );

    let out_atlas_tile_pose = out_atlas_tile_offset + tile_xy;
    let out_halton_jitter   = pack2x16float(halton_jitter);
    let out_color           = vec4<f32>(probe_irradiance, bitcast<f32>(out_halton_jitter));

    textureStore(ss_probe_out, out_atlas_tile_pose, out_color);
}