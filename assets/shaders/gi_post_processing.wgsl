#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0) var in_floor_texture:   texture_2d<f32>;
@group(1) @binding(1) var in_floor_sampler:   sampler;

@group(1) @binding(2) var in_walls_texture:   texture_2d<f32>;
@group(1) @binding(3) var in_walls_sampler:   sampler;

@group(1) @binding(4) var in_objects_texture: texture_2d<f32>;
@group(1) @binding(5) var in_objects_sampler: sampler;


@group(1) @binding(6) var in_irradiance_texture:         texture_2d<f32>;
@group(1) @binding(7) var in_irradiance_texture_sampler: sampler;

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
    let in_floor_diffuse   = textureSample(in_floor_texture,   in_floor_sampler, uv).xyz;
    let in_walls_diffuse   = textureSample(in_walls_texture,   in_walls_sampler, uv).xyz;
    let in_objects_diffuse = textureSample(in_objects_texture, in_objects_sampler, uv).xyz;

    let in_irradiance = textureSample(in_irradiance_texture, in_irradiance_texture_sampler, uv).xyz;



    let floor_final_rgb = in_floor_diffuse * lin_to_srgb(in_irradiance);


    let out = floor_final_rgb;

    return vec4<f32>(out, 1.0);
}