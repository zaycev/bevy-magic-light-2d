#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct GpuLightOmni
{
    color:     vec3<f32>,
    intensity: f32,
    r_max:     f32,
    center:    vec2<f32>,
}

struct GpuLightOmniBuffer
{
    count: u32,
    data:  array<GpuLightOmni>,
}

struct GpuGlobalParams
{
    screen_size:         vec2<f32>,
    screen_size_inv:     vec2<f32>,
    view_proj:           mat4x4<f32>,
    inverse_view_proj :  mat4x4<f32>,
}

@group(0) @binding(0) var                screen_texture:  texture_2d<f32>;
@group(0) @binding(1) var                texture_sampler: sampler;
@group(0) @binding(2) var<uniform>       globals:         GpuGlobalParams;
@group(0) @binding(3) var<storage, read> lights_omni:     GpuLightOmniBuffer;

fn F(r: f32, r_max: f32) -> f32
{
    let numerator = exp(-9.0 * r * r / (r_max * r_max)) - exp(-9.0);
    let denominator = 1.0 - exp(-9.0);
    return numerator / denominator;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32>
{
    let ndc = (in.uv * 2.0) - 1.0;
    let pose = (globals.inverse_view_proj * vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0)).xy;
    var color = textureSample(screen_texture, texture_sampler, in.uv);

    var total_irradiance = vec3<f32>(0.0);
    for (var i: u32 = 0; i < lights_omni.count; i++)
    {
        let light = lights_omni.data[i];
        let r = length(pose - light.center);
        let a = F(r, light.r_max);
        total_irradiance += a * light.color * light.intensity;
    }

    let out = color.xyz * 0.01 + color.xyz * total_irradiance;

    return vec4<f32>(out, 1.0);
}