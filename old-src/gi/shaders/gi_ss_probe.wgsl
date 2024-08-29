#import bevy_magic_light_2d::gi_types::{LightPassParams, ProbeDataBuffer, SkylightMaskBuffer, LightSourceBuffer}
#import bevy_magic_light_2d::gi_math
#import bevy_magic_light_2d::gi_camera::{CameraParams, world_to_sdf_uv, bilinear_sample_rgba, screen_to_world, world_to_ndc, ndc_to_screen, bilinear_sample_r}
#import bevy_magic_light_2d::gi_attenuation::light_attenuation_r_two
#import bevy_magic_light_2d::gi_halton::hammersley2d
#import bevy_magic_light_2d::gi_raymarch::{raymarch_bounce, raymarch_primary}

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<uniform> cfg:                   LightPassParams;
@group(0) @binding(2) var<storage> probes:                ProbeDataBuffer;
@group(0) @binding(3) var<storage> skylight_masks_buffer: SkylightMaskBuffer;
@group(0) @binding(4) var<storage> lights_source_buffer:  LightSourceBuffer;
@group(0) @binding(5) var          sdf_in:                texture_2d<f32>;
@group(0) @binding(6) var          sdf_in_sampler:        sampler;
@group(0) @binding(7) var          ss_probe_out:          texture_storage_2d<rgba16float, write>;


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let reservoir_size           = i32(cfg.reservoir_size);
    let probe_size_f32           = f32(cfg.probe_size);
    let frames_max               = cfg.probe_size * cfg.probe_size;
    let frame_index              = cfg.frame_counter % reservoir_size;
    let halton_jitter            = hammersley2d(frame_index, reservoir_size);
    let probe_tile_origin_screen = tile_xy * cfg.probe_size;

    // Get current frame.
    let probe_offset_world  = halton_jitter * probe_size_f32;
    let probe_center_world_unbiased = screen_to_world(
                                              probe_tile_origin_screen,
                                              camera_params.screen_size,
                                              camera_params.inverse_view_proj,
                                              camera_params.screen_size_inv,
                                          );
    let probe_center_world  =  probe_center_world_unbiased + probe_offset_world;

    let probe_ndc    = world_to_ndc(probe_center_world, camera_params.view_proj);
    let probe_screen = ndc_to_screen(probe_ndc, camera_params.screen_size);
    var is_masked    = 1.0;

    // Check if the probe is masked from skylight.
    for (var i: i32 = 0; i < i32(skylight_masks_buffer.count); i++) {
        let mask = skylight_masks_buffer.data[i];
        if probe_center_world.x > mask.center.x - mask.h_extent.x &&
           probe_center_world.x < mask.center.x + mask.h_extent.x &&
           probe_center_world.y > mask.center.y - mask.h_extent.y &&
           probe_center_world.y < mask.center.y + mask.h_extent.y {
            is_masked = 0.0;
            break;
        }
    }

    var probe_irradiance = vec3<f32>(0.0);

    let uv = world_to_sdf_uv(probe_center_world_unbiased, camera_params.view_proj, camera_params.inv_sdf_scale);
    let dist = bilinear_sample_r( sdf_in, sdf_in_sampler, uv);
    if dist > 0.0 {

        let skylight = cfg.skylight_color * is_masked;;

        // Compute direct irradiance from lights in the current frame.
        probe_irradiance = vec3<f32>(skylight);
        for (var i: i32 = 0; i < i32(lights_source_buffer.count); i++) {

            let light = lights_source_buffer.data[i];

            let ray_result = raymarch_primary(
                probe_center_world,
                light.center,
                32,
                sdf_in,
                sdf_in_sampler,
                camera_params,
                0.3
            );

            let att = light_attenuation_r_two(
                probe_center_world,
                light.center,
                light.falloff.x,
                light.falloff.y,
                light.falloff.z,
            );

            if (ray_result.success > 0) {
                probe_irradiance += light.color * att * light.intensity;
            }
        }

    }

    // Coordinates of the screen-space cache output tile.
    let atlas_row  = frame_index / cfg.probe_size;
    let atlas_col  = frame_index % cfg.probe_size;

    let out_atlas_tile_offset = vec2<i32>(
        cfg.probe_atlas_cols * atlas_col,
        cfg.probe_atlas_rows * atlas_row,
    );

    let out_atlas_tile_pose = out_atlas_tile_offset + tile_xy;
    let out_halton_jitter   = pack2x16float(halton_jitter);
    let out_color           = vec4<f32>(probe_irradiance, bitcast<f32>(out_halton_jitter));

    textureStore(ss_probe_out, out_atlas_tile_pose, out_color);
}