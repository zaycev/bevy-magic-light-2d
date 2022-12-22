#define_import_path bevy_2d_gi_experiment::gi_camera

struct CameraParams {
    screen_size:         vec2<f32>,
    screen_size_inv:     vec2<f32>,
    view_proj:           mat4x4<f32>,
    inverse_view_proj :  mat4x4<f32>,
}

fn screen_to_ndc(
    screen_pose:     vec2<i32>,
    screen_size:     vec2<f32>,
    screen_size_inv: vec2<f32>) -> vec2<f32> {
    let screen_pose_f32 = vec2<f32>(0.0, screen_size.y)
                        + vec2<f32>(f32(screen_pose.x), f32(-screen_pose.y));
    return (screen_pose_f32 * screen_size_inv) * 2.0 - 1.0;
}

fn ndc_to_screen(ndc: vec2<f32>, screen_size: vec2<f32>) -> vec2<i32> {
    let screen_pose_f32 = (ndc + 1.0) * 0.5 * screen_size;
    return vec2<i32>(
        i32(screen_pose_f32.x),
        i32(screen_size.y - screen_pose_f32.y),
    );
}

fn screen_to_world(
    screen_pose:       vec2<i32>,
    screen_size:       vec2<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_size_inv:   vec2<f32>) -> vec2<f32> {
    return (inverse_view_proj * vec4<f32>(screen_to_ndc(screen_pose, screen_size, screen_size_inv), 0.0, 1.0)).xy;
}

fn world_to_ndc(
    world_pose:  vec2<f32>,
    view_proj:   mat4x4<f32>) -> vec2<f32> {
    return (view_proj * vec4<f32>(world_pose, 0.0, 1.0)).xy;
}

fn world_to_screen(
    world_pose:  vec2<f32>,
    screen_size: vec2<f32>,
    view_proj:   mat4x4<f32>) -> vec2<i32> {
    return ndc_to_screen(world_to_ndc(world_pose, view_proj), screen_size);
}
