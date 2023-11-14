#define_import_path bevy_magic_light_2d::gi_raymarch

#import bevy_magic_light_2d::gi_math::{fast_normalize_2d, distance_squared, hash}
#import bevy_magic_light_2d::gi_camera::{CameraParams, sdf_uv_to_world, world_to_sdf_uv, bilinear_sample_r}

struct RayMarchResult {
    success:  i32,      //
    step: i32,          // steps
    pose: vec2<f32>,    // curr spot
}

fn raymarch(
    in_ray_origin:      vec2<f32>,
    in_ray_target:      vec2<f32>,
    max_steps:          i32,
    sdf:                texture_2d<f32>,
    sdf_sampler:        sampler,
    camera_params:      CameraParams,
    rm_jitter_contrib:  f32,
) -> RayMarchResult {

    var ray_target  = in_ray_target;
    var ray_origin  = in_ray_origin;
    let target_uv   = world_to_sdf_uv(ray_target, camera_params.view_proj, camera_params.inv_sdf_scale);
    let target_dist = bilinear_sample_r(sdf, sdf_sampler, target_uv);

    if (target_dist < 0.0) {
        let t = ray_target;
        ray_target = ray_origin;
        ray_origin = t;
    }

    let ray_direction          = fast_normalize_2d(ray_target - ray_origin);
    let stop_at                = distance_squared(ray_origin, ray_target);

    var ray_progress:   f32    = 0.0;
    var h                      = vec2<f32>(0.0);
    var h_prev                 = h;
    let min_sdf                = 1e-4;
    var inside                 = true;
    let max_inside_dist        = 20.0;
    let max_inside_dist_sq     = max_inside_dist * max_inside_dist;

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
            let jitter = hash(h);
            ray_progress += ray_travel * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * ray_travel * jitter;
        } else {
            ray_progress += ray_travel;
        }
    }

    return RayMarchResult(0, max_steps, h);
}

fn raymarch_primary(
    in_ray_origin:      vec2<f32>,
    in_ray_target:      vec2<f32>,
    max_steps:          i32,
    sdf:                texture_2d<f32>,
    sdf_sampler:        sampler,
    camera_params:      CameraParams,
    rm_jitter_contrib:  f32,
) -> RayMarchResult {

    var ray_target  = in_ray_target;
    var ray_origin  = in_ray_origin;

    let ray_direction          = normalize(ray_target - ray_origin);
    let stop_at                = distance_squared(ray_origin, ray_target);

    var ray_progress:   f32    = 0.0;
    var h                      = vec2<f32>(0.0);
    var h_prev                 = h;
    let min_sdf                = 1e-4;

    for (var i: i32 = 0; i < max_steps; i++) {

        h_prev = h;
        h = ray_origin + ray_progress * ray_direction;

        if ray_progress * ray_progress >= stop_at {
            return RayMarchResult(1, i, h_prev);
        }


        let uv = world_to_sdf_uv(h, camera_params.view_proj, camera_params.inv_sdf_scale);
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            return RayMarchResult(0, i, h_prev);
        }

        let scene_dist = bilinear_sample_r(sdf, sdf_sampler, uv);
        if scene_dist <= min_sdf {
            return RayMarchResult(0, i, h);
        }

        let ray_travel = max(abs(scene_dist), 0.0);

        ray_progress += ray_travel * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * ray_travel * hash(h);
   }

    return RayMarchResult(0, max_steps, h);
}


fn raymarch_bounce(
    in_ray_origin:      vec2<f32>,
    in_ray_target:      vec2<f32>,
    max_steps:          i32,
    sdf:                texture_2d<f32>,
    sdf_sampler:        sampler,
    camera_params:      CameraParams,
    rm_jitter_contrib:  f32,
) -> RayMarchResult {

    var ray_target  = in_ray_target;
    var ray_origin  = in_ray_origin;

    let target_uv   = world_to_sdf_uv(ray_target, camera_params.view_proj, camera_params.inv_sdf_scale);
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
    let min_sdf                = 1e-4;

    for (var i: i32 = 0; i < max_steps; i++) {

        h_prev = h;
        h = ray_origin + ray_progress * ray_direction;

        if ray_progress * ray_progress >= stop_at {
            return RayMarchResult(1, i, h_prev);
        }

        let uv = world_to_sdf_uv(h, camera_params.view_proj, camera_params.inv_sdf_scale);
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            return RayMarchResult(0, i, h_prev);
        }

        let scene_dist = bilinear_sample_r(sdf, sdf_sampler, uv);
        if  scene_dist <= min_sdf {
            return RayMarchResult(0, i, h);
        }

        let ray_travel = max(abs(scene_dist), 0.5);

        ray_progress += ray_travel * (1.0 - rm_jitter_contrib)
                      + rm_jitter_contrib * ray_travel * hash(h);
    }

    return RayMarchResult(0, max_steps, h);
}