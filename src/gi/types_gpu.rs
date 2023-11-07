use bevy::math::Vec3Swizzles;
use bevy::prelude::{Mat4, Transform, Vec2, Vec3, Vec4};
use bevy::render::render_resource::ShaderType;

use crate::gi::constants::GI_SCREEN_PROBE_SIZE;
use crate::gi::types::OmniLightSource2D;

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct GpuOmniLightSource {
    pub center:    Vec2,
    pub intensity: f32,
    pub color:     Vec3,
    pub falloff:   Vec3,
}

impl GpuOmniLightSource
{
    pub fn new(light: OmniLightSource2D, center: Vec2) -> Self
    {
        let color = light.color.as_rgba_f32();
        Self {
            center,
            intensity: light.intensity,
            color: Vec3::new(color[0], color[1], color[2]),
            falloff: light.falloff,
        }
    }
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct GpuLightSourceBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuOmniLightSource>,
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct GpuLightOccluder2D {
    pub center: Vec2,
    pub rotation: Vec4,
    pub h_extent: Vec2,
}

impl GpuLightOccluder2D
{
    pub fn new(transform: &Transform, h_extent: Vec2) -> Self
    {
        let center = transform.translation.xy();
        let rotation = transform.rotation.inverse().into();
        // let h_extent = h_extent * transform.scale.xy();
        Self {
            center,
            rotation,
            h_extent,
        }
    }
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct GpuLightOccluderBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuLightOccluder2D>,
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct GpuCameraParams {
    pub screen_size:       Vec2,
    pub screen_size_inv:   Vec2,
    pub view_proj:         Mat4,
    pub inverse_view_proj: Mat4,
    pub sdf_scale:         Vec2,
    pub inv_sdf_scale:     Vec2,
}

#[rustfmt::skip]
#[derive(Clone, ShaderType, Debug)]
pub struct GpuLightPassParams {
    pub frame_counter:          i32,
    pub probe_size:             i32,
    pub probe_atlas_cols:       i32,
    pub probe_atlas_rows:       i32,
    pub skylight_color:         Vec3,

    pub reservoir_size:              u32,
    pub smooth_kernel_size_h:        u32,
    pub smooth_kernel_size_w:        u32,
    pub direct_light_contrib:        f32,
    pub indirect_light_contrib:      f32,
    pub indirect_rays_per_sample:    i32,
    pub indirect_rays_radius_factor: f32,
}

impl Default for GpuLightPassParams
{
    fn default() -> Self
    {
        Self {
            frame_counter:    0,
            probe_size:       0,
            probe_atlas_cols: 0,
            probe_atlas_rows: 0,
            skylight_color:   Vec3::new(0.003, 0.0078, 0.058) / 100.0,

            reservoir_size:         16,
            smooth_kernel_size_h:   2,
            smooth_kernel_size_w:   1,
            direct_light_contrib:   0.2,
            indirect_light_contrib: 0.8,

            indirect_rays_per_sample:    64,
            indirect_rays_radius_factor: 3.0,
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, ShaderType, Default)]
pub struct GpuProbeData {
    pub camera_pose: Vec2,
}

#[rustfmt::skip]
#[derive(Clone, ShaderType)]
pub struct GpuProbeDataBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuProbeData>,
}

impl Default for GpuProbeDataBuffer
{
    fn default() -> Self
    {
        const MAX_PROBES: u32 = (GI_SCREEN_PROBE_SIZE * GI_SCREEN_PROBE_SIZE) as u32;
        Self {
            count: MAX_PROBES,
            data:  vec![
                GpuProbeData {
                    camera_pose: Vec2::ZERO,
                };
                MAX_PROBES as usize
            ],
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, ShaderType, Default)]
pub struct GpuSkylightMaskData {
    pub center:   Vec2,
    pub h_extent: Vec2,
}

impl GpuSkylightMaskData
{
    pub fn new(center: Vec2, h_extent: Vec2) -> Self
    {
        Self { center, h_extent }
    }
}

#[rustfmt::skip]
#[derive(Clone, ShaderType, Default)]
pub struct GpuSkylightMaskBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data: Vec<GpuSkylightMaskData>,
}
