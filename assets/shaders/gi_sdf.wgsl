#import bevy_magic_light_2d::gi_math
#import bevy_magic_light_2d::gi_types
#import bevy_magic_light_2d::gi_camera

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
     let texel_pos  = vec2<i32>(invocation_id.xy);
     let dims = textureDimensions(sdf_out);
     let uv = (vec2<f32>(texel_pos) + 0.5) / vec2<f32>(dims);
    //  let uv = vec2<f32>(uv.x, 1.0 - uv.y);

     let world_pose = sdf_uv_to_world(uv, 
        camera_params.inverse_view_proj,
        camera_params.sdf_scale);

     let sdf_min      = 0.9;
     var sdf_merged   = sdf_aabb_occluder(world_pose.xy, 0);
     for (var i: i32 = 1; i < i32(light_occluder_buffer.count); i++) {
         sdf_merged = round_merge(sdf_merged, sdf_aabb_occluder(world_pose.xy, i), sdf_min);
     }

     var sdf = clamp(sdf_merged, sdf_min, 1e+4);

     textureStore(sdf_out, texel_pos, vec4<f32>(sdf, 0.0, 0.0, 0.0));
}