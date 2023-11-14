#import bevy_magic_light_2d::gi_types::LightOccluderBuffer
#import bevy_magic_light_2d::gi_math::quat_mul
#import bevy_magic_light_2d::gi_camera::{CameraParams, sdf_uv_to_world}

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<storage> light_occluder_buffer: LightOccluderBuffer;
@group(0) @binding(2) var          sdf_out:               texture_storage_2d<r16float, read_write>;

fn sdf_aabb_occluder(p: vec2<f32>, occluder_i: i32) -> f32 {
    let occluder = light_occluder_buffer.data[occluder_i];
    let local_p = quat_mul(occluder.rotation, vec3<f32>(occluder.center - p, 0.0)).xy;
    let d        = abs(local_p) - occluder.h_extent;
    let d_max    = max(d, vec2<f32>(0.0));
    let d_o      = length(d_max);
    let d_i      = min(max(d.x, d.y), 0.0);
    return d_o + d_i;
}

fn round_merge(s1: f32, s2: f32, r: f32) -> f32 {
    var intersection_space = vec2<f32>(s1 - r, s1 - r); // s1, s1 is intended
        intersection_space = min(intersection_space, vec2<f32>(0.0));
    let inside_distance    = -length(intersection_space);
    let simple_union       = min(s1, s2);
    let outside_distance   = max(simple_union, r);
    return inside_distance + outside_distance;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
     let texel_pos  = vec2<i32>(invocation_id.xy);
     let dims = textureDimensions(sdf_out);
     let uv = (vec2<f32>(texel_pos) + 0.5) / vec2<f32>(dims);

     let world_pose = sdf_uv_to_world(uv,
        camera_params.inverse_view_proj,
        camera_params.sdf_scale);
    let r = 1.2;

     var sdf_merged   = round_merge(
        1e+10,
        sdf_aabb_occluder(world_pose.xy, 0),
        r,
     );
     for (var i: i32 = 1; i < i32(light_occluder_buffer.count); i++) {
        sdf_merged = round_merge(sdf_merged, sdf_aabb_occluder(world_pose.xy, i), r);
     }

    textureStore(sdf_out, texel_pos, vec4<f32>(sdf_merged, 0.0, 0.0, 0.0));
}