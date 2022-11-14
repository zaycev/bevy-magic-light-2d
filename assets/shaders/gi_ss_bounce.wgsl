#import bevy_2d_gi_experiment::gi_math
#import bevy_2d_gi_experiment::gi_types
#import bevy_2d_gi_experiment::gi_camera
#import bevy_2d_gi_experiment::gi_halton
#import bevy_2d_gi_experiment::gi_attenuation

@group(0) @binding(0) var<uniform> camera_params:     CameraParams;
@group(0) @binding(1) var<uniform> state:             GiState;
@group(0) @binding(2) var          sdf_in:            texture_storage_2d<r16float,    read>;
@group(0) @binding(3) var          ss_probe_in:       texture_storage_2d<rgba16float, read>;
@group(0) @binding(4) var          ss_bounce_out:     texture_storage_2d<rgba32float, write>;


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
    let rm_jitter_contrub: f32 = 0.1;
    let rm_sdf_contrib:    f32 = 0.0;

    let ray_direction          = normalize(light_pose - ray_origin);
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
                return light_contrib * 0.5 + 0.5;
            }
        }

        scene_dist = new_scene_dist;
        if (scene_dist <= 0.0) {
            return 0.0;
        }

        light_contrib = min(light_contrib, scene_dist / ray_progress);

        // Jitter step.
        let jitter = radical_inverse_vdc(i);
        ray_progress += scene_dist * (1.0 - rm_jitter_contrub) + rm_jitter_contrub * scene_dist * jitter;
    }

    return 0.0;
}


@compute @workgroup_size(5, 5, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let reservoir_size           = 8;
    let probe_size_f32           = f32(state.ss_probe_size);
    let probe_cols               = state.ss_atlas_cols;
    let probe_rows               = state.ss_atlas_rows;
    let frames_max               = state.ss_probe_size * state.ss_probe_size;
    let frame_index              = state.gi_frame_counter % reservoir_size;
    let halton                   = hammersley2d(frame_index, reservoir_size);
    let probe_tile_origin_screen = tile_xy * state.ss_probe_size;

    let atlas_row = frame_index / state.ss_probe_size;
    let atlas_col = frame_index % state.ss_probe_size;

    let probe_cols               = state.ss_atlas_cols;
    let probe_rows               = state.ss_atlas_rows;

    let out_atlas_tile_offset = vec2<i32>(
        state.ss_atlas_cols * atlas_col,
        state.ss_atlas_rows * atlas_row,
    );

    let out_atlas_tile_pose = out_atlas_tile_offset + tile_xy;

    let probe             = textureLoad(ss_probe_in, out_atlas_tile_pose);
    let direct_irradiance = probe.xyz;
    var total_irradiance  = direct_irradiance;
    let probe_size_f32    = f32(state.ss_probe_size);
    let halton            = unpack2x16float(bitcast<u32>(probe.w));
    let probe_tile_origin_screen = tile_xy * state.ss_probe_size;

    let probe_offset_world  = halton * probe_size_f32;
    let probe_center_world  = screen_to_world(
        probe_tile_origin_screen,
        camera_params.screen_size,
        camera_params.inverse_view_proj) + probe_offset_world;

    // Compute indirrect light.
    let pi                  = radians(180.0);
    let rays_per_sample     = 28;
    let golden_angle        = (2.0 * pi) / f32(rays_per_sample);
    var indirect_irradiance = vec3<f32>(0.0);
    var total_rays          = 0;
    var total_w             = 0.0;
    var h                   = hash(probe_center_world);

    let light_a = 800.0;
    let light_b = 200.0;
    let light_c = 0.01;

    let r_bias = 12.0;
    let r_step = 16.0;

    for (var k = 1; k <= 6; k++) {

        var r = r_bias + f32(k * k) * r_step;
            r = 0.3 * r + 0.7 * r * (0.5 - h);

         for (var ray_i = 0; ray_i < rays_per_sample; ray_i++) {

            var base_angle  = golden_angle * f32(ray_i);
                base_angle += radians(180.0) * (0.8 - h);

            var sample_world = probe_center_world + vec2<f32>(r) * normalize(vec2<f32>(
                cos(base_angle),
                sin(base_angle),
            ));

            total_rays += 1;

            let sample_screen = world_to_screen(
                sample_world,
                camera_params.screen_size,
                camera_params.view_proj);

            let sample_tile_pose = sample_screen / state.ss_probe_size;
            let sample_atlas_pose = out_atlas_tile_offset + sample_tile_pose;


            let sample_kernel  = 0;
            let sample_probe   = textureLoad(ss_probe_in, sample_atlas_pose);
            var sample_xyz     = sample_probe.xyz;


            // sample_xyz = sample_xyz
            //            + textureLoad(ss_probe_in, sample_atlas_pose + vec2<i32>( 0,  1)).xyz
            //            + textureLoad(ss_probe_in, sample_atlas_pose + vec2<i32>( 0, -1)).xyz
            //            + textureLoad(ss_probe_in, sample_atlas_pose + vec2<i32>( 1,  0)).xyz
            //            + textureLoad(ss_probe_in, sample_atlas_pose + vec2<i32>(-1,  0)).xyz;
            // sample_xyz = sample_xyz / 5.0;


            let sample_halton       = unpack2x16float(bitcast<u32>(sample_probe.w));
            let sample_offset_world = sample_halton * probe_size_f32;

            sample_world           += sample_offset_world;


            let raymarch_sample_to_probe = raymarch(
                probe_center_world,
                sample_world,
                32,
            );

            // Discard if ocluded.
            if raymarch_sample_to_probe <= 0.0 {
                continue;
            }

            // Discrad if sample offscreen.
            let sample_ndc = world_to_ndc(sample_world, camera_params.view_proj);
            if any(sample_ndc < vec2<f32>(-1.0)) || any(sample_ndc > vec2<f32>(1.0)) {
                continue;
            }

            let att = light_attenuation_r2(
                sample_world,
                probe_center_world,
                light_a,
                light_b,
                light_c,
            );

            let sample_irradiance = att * sample_xyz;
            indirect_irradiance  += sample_irradiance;
         }
    }

    // direct_irradiance
    indirect_irradiance = indirect_irradiance / f32(total_rays / 4);
    indirect_irradiance = clamp(indirect_irradiance, vec3<f32>(0.0), vec3<f32>(1.0));
    total_irradiance    = direct_irradiance + log(vec3<f32>(1.0) + indirect_irradiance);

    textureStore(ss_bounce_out, out_atlas_tile_pose, vec4(total_irradiance, probe.w));
}
