#import bevy_magic_light_2d::gi_types::LightPassParams
#import bevy_magic_light_2d::gi_math::fast_normalize_2d
#import bevy_magic_light_2d::gi_camera::{CameraParams, world_to_sdf_uv, bilinear_sample_rgba, screen_to_world, world_to_screen, world_to_ndc}
#import bevy_magic_light_2d::gi_halton::radical_inverse_vdc
#import bevy_magic_light_2d::gi_attenuation
#import bevy_magic_light_2d::gi_raymarch::raymarch_bounce

@group(0) @binding(0) var<uniform> camera_params:     CameraParams;
@group(0) @binding(1) var<uniform> cfg:               LightPassParams;
@group(0) @binding(2) var          sdf_in:            texture_2d<f32>;
@group(0) @binding(3) var          sdf_in_sampler:    sampler;
@group(0) @binding(4) var          ss_probe_in:       texture_storage_2d<rgba16float, read>;
@group(0) @binding(5) var          ss_bounce_out:     texture_storage_2d<rgba32float, write>;


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let reservoir_size           = i32(cfg.reservoir_size);
    let frame_index              = cfg.frame_counter % reservoir_size;

    let atlas_row = frame_index / cfg.probe_size;
    let atlas_col = frame_index % cfg.probe_size;

    let out_atlas_tile_offset = vec2<i32>(
        cfg.probe_atlas_cols * atlas_col,
        cfg.probe_atlas_rows * atlas_row,
    );

    let out_atlas_tile_pose = out_atlas_tile_offset + tile_xy;

    let probe             = textureLoad(ss_probe_in, out_atlas_tile_pose);
    let direct_irradiance = probe.xyz;
    var total_irradiance  = direct_irradiance;
    let probe_size_f32    = f32(cfg.probe_size);
    let halton            = unpack2x16float(bitcast<u32>(probe.w));
    let probe_tile_origin_screen = tile_xy * cfg.probe_size;

    let probe_offset_world  = halton * probe_size_f32;
    let probe_center_world  = screen_to_world(
        probe_tile_origin_screen,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    ) + probe_offset_world;

    // Compute indirrect light.
    let mm                   = 0.7639320225; // Magic number.
    let pi                   = radians(180.0);
    let pi2                  = pi * 2.0;
    var indirect_irradiance  = vec3<f32>(0.0);
    var total_rays           = 0;
    var rays_per_sample      = cfg.indirect_rays_per_sample;
    let golden_angle         = pi * mm;

    var r_bias = 4.0;
    var r_step = 16.0;
    var hh = radical_inverse_vdc(frame_index) / f32(reservoir_size);

    {
        r_step *= mm;
        r_step  = r_step + r_step * (0.5 - r_step * hh);
    }

    let k_max  = 2;
    let jitter = 0.5;

    for (var k = 1; k <= k_max; k++) {

        let angle_bias   = pi2 * f32(k) / f32(k_max);

        var r = r_bias + f32(pow(cfg.indirect_rays_radius_factor, f32(k))) * r_step;
            r = r + r * hh * jitter;

        for (var ray_i = 0; ray_i < rays_per_sample; ray_i++) {

            total_rays += 1;

            var base_angle  = angle_bias + golden_angle * f32(ray_i);
                base_angle += radians(360.0) * hh;

            var sample_world = probe_center_world + vec2<f32>(r) * fast_normalize_2d(vec2<f32>(
                cos(base_angle),
                sin(base_angle),
            ));

            var raymarch_sample_to_probe = raymarch_bounce(
                probe_center_world,
                sample_world,
                32,
                sdf_in,
                sdf_in_sampler,
                camera_params,
                0.3
            );

            if raymarch_sample_to_probe.success <= 0 {
                continue;
            }

            let sample_screen = world_to_screen(
                raymarch_sample_to_probe.pose,
                camera_params.screen_size,
                camera_params.view_proj);

            let sample_tile_pose = sample_screen / cfg.probe_size;
            let sample_atlas_pose = out_atlas_tile_offset + sample_tile_pose;

            let sample_kernel  = 0;
            let sample_probe   = textureLoad(ss_probe_in, sample_atlas_pose);
            let sample_xyz     = sample_probe.xyz;

            let sample_halton       = unpack2x16float(bitcast<u32>(sample_probe.w));
            let sample_offset_world = sample_halton * probe_size_f32;

            sample_world           += sample_offset_world;

            // Discrad if sample offscreen.
            let sample_ndc = world_to_ndc(sample_world, camera_params.view_proj);
            if any(sample_ndc < vec2<f32>(-1.0, -1.0)) || any(sample_ndc > vec2<f32>(1.0, 1.0)) {
                continue;
            }

            let sample_irradiance = sample_xyz;
            indirect_irradiance  += sample_irradiance * 0.6; // 0.4 is absorbed by surface.
        }
    }

    indirect_irradiance = indirect_irradiance / f32(total_rays / k_max);
    total_irradiance  = cfg.indirect_light_contrib * indirect_irradiance
                      + cfg.direct_light_contrib   * direct_irradiance;

    textureStore(ss_bounce_out, out_atlas_tile_pose, vec4(total_irradiance, probe.w));
}
