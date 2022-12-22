#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var in_diffuse_texture: texture_2d<f32>;
@group(1) @binding(1)
var in_diffuse_sampler: sampler;
@group(1) @binding(2)
var in_irradiance_texture: texture_2d<f32>;
@group(1) @binding(3)
var in_irradiance_texture_sampler: sampler;

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
    let diffuse = textureSample(in_diffuse_texture, in_diffuse_sampler, uv).xyz;
    let irradiance = textureSample(in_irradiance_texture, in_irradiance_texture_sampler, uv).xyz;
    let out_color = diffuse * lin_to_srgb(irradiance);
    return vec4<f32>(out_color, 1.0);
}