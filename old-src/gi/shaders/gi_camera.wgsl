#define_import_path bevy_magic_light_2d::gi_camera

struct CameraParams {
    screen_size:         vec2<f32>,
    screen_size_inv:     vec2<f32>,
    view_proj:           mat4x4<f32>,
    inverse_view_proj :  mat4x4<f32>,
    sdf_scale: vec2<f32>,
    inv_sdf_scale: vec2<f32>,
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

fn world_to_sdf_uv(world_pose: vec2<f32>, view_proj: mat4x4<f32>, inv_sdf_scale: vec2<f32>) -> vec2<f32> {
    let ndc = world_to_ndc(world_pose, view_proj);
    let ndc_sdf = ndc * inv_sdf_scale;
    let uv = (ndc_sdf + 1.0) * 0.5;
    let y = 1.0 - uv.y;
    return vec2<f32>(uv.x, y);
}

fn sdf_uv_to_world(uv_in: vec2<f32>, inverse_view_proj: mat4x4<f32>, sdf_scale: vec2<f32>) -> vec2<f32> {
    let y = 1.0 - uv_in.y;
    let uv = vec2<f32>(uv_in.x, y);
    let ndc_sdf = (uv * 2.0) - 1.0;
    let ndc = ndc_sdf * sdf_scale;
    return (inverse_view_proj * vec4<f32>(ndc, 0.0, 1.0)).xy;
}

fn bilinear_filter(texels: vec4<f32>, scaled_uv: vec2<f32>) -> f32 {
    // texels.x = -u, +v
    // texels.y = +u, +v,
    // texels.z = +u, -v,
    // texels.w = -u, -v
    let f = fract(scaled_uv - 0.5);
    return mix(mix(texels.w, texels.z, f.x), mix(texels.x, texels.y, f.x), f.y);
}

fn bilinear_sample_r(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> f32 {
    let texels = textureGather(0, t, s, uv);
    let dims = textureDimensions(t);
    let scaled_uv = uv * vec2<f32>(dims);
    return bilinear_filter(texels, scaled_uv);
}

fn bilinear_sample_rgba(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec4<f32> {
    let dims = textureDimensions(t);
    let scaled_uv = uv * vec2<f32>(dims);
    
    let r_texels = textureGather(0, t, s, uv);
    let r = bilinear_filter(r_texels, scaled_uv);
    let g_texels = textureGather(1, t, s, uv);
    let g = bilinear_filter(g_texels, scaled_uv);
    let b_texels = textureGather(2, t, s, uv);
    let b = bilinear_filter(b_texels, scaled_uv);
    let a_texels = textureGather(3, t, s, uv);
    let a = bilinear_filter(a_texels, scaled_uv);
    return vec4<f32>(r, g, b, a);
}