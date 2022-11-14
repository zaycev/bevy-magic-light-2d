#import bevy_2d_gi_experiment::gi_math
#import bevy_2d_gi_experiment::gi_types
#import bevy_2d_gi_experiment::gi_camera

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<storage> light_occluder_buffer: LightOccluderBuffer;
@group(0) @binding(2) var          sdf_out:               texture_storage_2d<r16float, write>;

fn sdf_aabb_occluder(p: vec2<f32>, occluder_i: i32) -> f32 {
    let occluder = light_occluder_buffer.data[occluder_i];
    let d        = abs(p - occluder.center) - occluder.h_extent;
    return length(vec2<f32>(
        max(d.x, 0.0),
        max(d.y, 0.0),
    ));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
     let screen_pose  = vec2<i32>(invocation_id.xy);
     let world_pose   = screen_to_world(screen_pose, camera_params.screen_size, camera_params.inverse_view_proj);

     var sdf_min      = sdf_aabb_occluder(world_pose.xy, 0);
     for (var i: i32 = 1; i < i32(light_occluder_buffer.count); i++) {
         sdf_min = min(sdf_min, sdf_aabb_occluder(world_pose.xy, i));
     }
     textureStore(sdf_out, screen_pose, vec4<f32>(sdf_min, 0.0, 0.0, 0.0));
}