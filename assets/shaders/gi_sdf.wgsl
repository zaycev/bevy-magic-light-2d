#import bevy_2d_gi_experiment::gi_math
#import bevy_2d_gi_experiment::gi_types
#import bevy_2d_gi_experiment::gi_camera

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<storage> light_occluder_buffer: LightOccluderBuffer;
@group(0) @binding(2) var          sdf_out:               texture_storage_2d<r16float, read_write>;


fn sdf_aabb_occluder(p: vec2<f32>, occluder_i: i32) -> f32 {
    let occluder = light_occluder_buffer.data[occluder_i];
    let d        = abs(p - occluder.center) - occluder.h_extent;
    let d_max    = max(d, vec2<f32>(0.0));
    let d_o      = fast_length_2d(d_max);
    let d_i      = min(max(d.x, d.y), 0.0);
    return d_o + d_i;
}

fn round_merge(s1: f32, s2: f32, r: f32) -> f32 {
    var intersection_space = vec2<f32>(s1 - r, s1 - r);
        intersection_space = min(intersection_space, vec2<f32>(0.0));
    let inside_distance    = -fast_length_2d(intersection_space);
    let simple_union       = min(s1, s2);
    let outside_distance   = max(simple_union, r);
    return inside_distance + outside_distance;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
     let screen_pose  = vec2<i32>(invocation_id.xy);
     let world_pose   = screen_to_world(
        screen_pose,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
     );

     var sdf_min      = sdf_aabb_occluder(world_pose.xy, 0);
     var sdf_merged   = sdf_min;
     for (var i: i32 = 1; i < i32(light_occluder_buffer.count); i++) {
         let s = sdf_aabb_occluder(world_pose.xy, i);

         sdf_min = min(sdf_min, s);
         sdf_merged = round_merge(sdf_merged, s, 0.9);

     }

     let r = 8.0;
     var sdf = sdf_merged;
     if sdf_min >= -r && sdf_min <= 0.0 {
         sdf = sdf_merged;
     }


     textureStore(sdf_out, screen_pose, vec4<f32>(sdf, 0.0, 0.0, 0.0));
}