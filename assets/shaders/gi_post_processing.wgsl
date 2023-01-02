#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils
#import bevy_magic_light_2d::gi_camera

@group(1) @binding(0) var in_floor_texture:              texture_2d<f32>;
@group(1) @binding(1) var in_floor_sampler:              sampler;
@group(1) @binding(2) var in_walls_texture:              texture_2d<f32>;
@group(1) @binding(3) var in_walls_sampler:              sampler;
@group(1) @binding(4) var in_objects_texture:            texture_2d<f32>;
@group(1) @binding(5) var in_objects_sampler:            sampler;
@group(1) @binding(6) var in_irradiance_texture:         texture_2d<f32>;
@group(1) @binding(7) var in_irradiance_texture_sampler: sampler;
@group(1) @binding(8) var in_sdf_texture:                texture_2d<f32>;
@group(1) @binding(9) var in_sdf_texture_sampler:        sampler;
@group(1) @binding(10) var in_pose_texture:              texture_2d<f32>;
@group(1) @binding(11) var in_pose_texture_sampler:      sampler;

fn lin_to_srgb(color: vec3<f32>) -> vec3<f32> {
   let x = color * 12.92;
   let y = 1.055 * pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - vec3<f32>(0.055);
   var clr = color;
   clr.x = select(x.x, y.x, (color.x < 0.0031308));
   clr.y = select(x.y, y.y, (color.y < 0.0031308));
   clr.z = select(x.z, y.z, (color.z < 0.0031308));
   return clr;
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    // Read diffuse textures.
    let in_floor_diffuse   = textureSample(in_floor_texture,   in_floor_sampler, uv);
    let in_walls_diffuse   = textureSample(in_walls_texture,   in_walls_sampler, uv);
    let in_objects_diffuse = textureSample(in_objects_texture, in_objects_sampler, uv);

    let in_irradiance = textureSample(in_irradiance_texture, in_irradiance_texture_sampler, uv).xyz;
    let in_sdf_uv     = textureSample(in_pose_texture, in_pose_texture_sampler, uv).xy;
    let in_sdf        = bilinear_sample_r(in_sdf_texture, in_sdf_texture_sampler, in_sdf_uv);

    // Calculate object irradiance.
    var object_irradiance = in_irradiance;
    let k_size = 5;
    let k_width = 24;
    var samples = 0.0;
    for (var i = -k_size; i <= k_size; i++) {
        for (var j = -k_size; j <= k_size; j++) {
            let offset = vec2<f32>(f32(i * k_width), f32(j * k_width));
            let irradiance_uv = coords_to_viewport_uv(position.xy - offset, view.viewport);

            let sample_irradiance = textureSample(
                in_irradiance_texture,
                in_irradiance_texture_sampler,
                irradiance_uv
            ).xyz;

            object_irradiance = max(object_irradiance, sample_irradiance);

            samples += 1.0;
        }
    }

    // object_irradiance /= samples;
    // object_irradiance =  mix(object_irradiance, vec3<f32>(1.0), 0.0000001);

    let final_floor   = in_floor_diffuse.xyz   * lin_to_srgb(in_irradiance);
    var final_walls   = in_walls_diffuse.xyz   * lin_to_srgb(in_irradiance);
    let final_objects = in_objects_diffuse.xyz * lin_to_srgb(object_irradiance);

    var out = vec4<f32>(mix(final_floor.xyz, final_walls.xyz, 1.0 - step(length(final_walls.xyz), 0.001)), 1.0);
        out = vec4<f32>(mix(out.xyz, final_objects.xyz, 1.0 - step(length(final_objects.xyz), 0.001)), 1.0);

    // out = vec4<f32>(in_irradiance, 1.0);

    return out;
}