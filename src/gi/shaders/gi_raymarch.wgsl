#define_import_path bevy_magic_light_2d::gi_raymarch

// #import bevy_magic_light_2d::gi_math
// #import bevy_magic_light_2d::gi_camera


struct RayMarchResult {
    success:  i32,      //
    step: i32,          // steps
    pose: vec2<f32>,    // curr spot
}

fn raymarch(
    ray_origin:    vec2<f32>,
    ray_target:    vec2<f32>,
    max_steps:     i32,
    sdf: texture_2d<f32>,
    sdf_sampler: sampler,
    camera_params: CameraParams,
    rm_jitter_contrib: f32,
) -> RayMarchResult {
    var ray_origin = ray_origin;
    var ray_target = ray_target;
    let target_uv = world_to_sdf_uv(ray_target, camera_params.view_proj, camera_params.inv_sdf_scale);
    let target_dist = bilinear_sample_r(sdf, sdf_sampler, target_uv);
    if (target_dist < 0.0) {
        let temp = ray_target;
        ray_target = ray_origin;
        ray_origin = temp;
    }

    let ray_direction          = fast_normalize_2d(ray_target - ray_origin);
    let stop_at                = distance_squared(ray_origin, ray_target);

    var ray_progress:   f32    = 0.0;
    var h                      = vec2<f32>(0.0);
    var h_prev                 = h;
    let min_sdf                = 0.5;
    var inside = true;
    let max_inside_dist = 20.0;
    let max_inside_dist_sq = max_inside_dist * max_inside_dist;

    for (var i: i32 = 0; i < max_steps; i++) {

        h_prev = h;
        h = ray_origin + ray_progress * ray_direction;

        if ((ray_progress * ray_progress >= stop_at) || (inside && (ray_progress * ray_progress > max_inside_dist))) {
            return RayMarchResult(1, i, h_prev);
        }


        let uv = world_to_sdf_uv(h, camera_params.view_proj, camera_params.inv_sdf_scale);
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            return RayMarchResult(0, i, h_prev);
        }

        let scene_dist = bilinear_sample_r(sdf, sdf_sampler, uv);
        if ((scene_dist <= min_sdf && !inside)) {
            return RayMarchResult(0, i, h);
        }
        if (scene_dist > 0.0) {
            inside = false;
        }
        let ray_travel = max(abs(scene_dist), 0.5);
        if (rm_jitter_contrib > 0.0) {
            // Jitter step.
            let jitter = radical_inverse_vdc(i);
            ray_progress += ray_travel * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * ray_travel * jitter;
        } else {
            ray_progress += ray_travel;
        }
    }

    return RayMarchResult(0, max_steps, h);
}