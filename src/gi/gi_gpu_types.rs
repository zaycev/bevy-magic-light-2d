use crate::SCREEN_SIZE;
use crate::gi::gi_component::LightSource;
use bevy::prelude::{Mat4, Vec2, Vec3};
use bevy::render::render_resource::ShaderType;

use super::gi_config::{GI_SDF_JITTER_CONTRIB, GI_SDF_MAX_STEPS, GI_SCREEN_PROBE_SIZE};

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GiGpuLightSource {
    pub(crate) center:    Vec2,
    pub(crate) intensity: f32,
    pub(crate) color:     Vec3,
    pub(crate) radius:    f32,
    pub(crate) falloff:   Vec3,
}

impl GiGpuLightSource {
    pub fn new(light: LightSource, center: Vec2) -> Self {
        let color = light.color.as_rgba_f32();
        Self {
            center:    center,
            radius:    light.radius,
            intensity: light.intensity,
            color:     Vec3::new(color[0], color[1], color[2]),
            falloff:   light.falloff,
        }
    }
}

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GiGpuLightSourceBuffer {
    pub(crate) count: u32,
    #[size(runtime)]
    pub(crate) data: Vec<GiGpuLightSource>,
}

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GiGpuLightOccluder {
    pub(crate) center: Vec2,
    pub(crate) h_extent: Vec2,
}

impl GiGpuLightOccluder {
    pub fn new(center: Vec2, h_extent: Vec2) -> Self {
        Self { center, h_extent }
    }
}

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GiGpuLightOccluderBuffer {
    pub(crate) count: u32,
    #[size(runtime)]
    pub(crate) data: Vec<GiGpuLightOccluder>,
}

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GiGpuCameraParams {
    pub(crate) screen_size:       Vec2,
    pub(crate) screen_size_inv:   Vec2,
    pub(crate) view_proj:         Mat4,
    pub(crate) inverse_view_proj: Mat4,
}

#[derive(Clone, ShaderType, Debug)]
pub(crate) struct GiGpuState {
    pub gi_frame_counter:   i32,
    pub ss_probe_size:      i32,
    pub ss_atlas_cols:      i32,
    pub ss_atlas_rows:      i32,
    pub sdf_max_steps:      i32,
    pub sdf_jitter_contrib: f32,
    pub gi_ambient:         Vec3,
}

impl Default for GiGpuState {
    fn default() -> Self {
        Self {
            gi_frame_counter: 0,
            ss_probe_size: 0,
            ss_atlas_cols: 0,
            ss_atlas_rows: 0,
            sdf_max_steps: GI_SDF_MAX_STEPS,
            sdf_jitter_contrib: GI_SDF_JITTER_CONTRIB,
            gi_ambient: Vec3::new(0.003, 0.0078, 0.058) / 100.0,
        }
    }
}

#[derive(Clone, ShaderType, Default)]
pub struct GiGpuProbeData {
    pub(crate) camera_pose: Vec2,
}

#[derive(Clone, ShaderType)]
pub struct GiGpuProbeDataBuffer {
    pub(crate) count: u32,
    #[size(runtime)]
    pub(crate) data: Vec<GiGpuProbeData>,
}

impl Default for GiGpuProbeDataBuffer {
    fn default() -> Self {
        let cols = SCREEN_SIZE.0 / (GI_SCREEN_PROBE_SIZE as usize);
        let rows = SCREEN_SIZE.1 / (GI_SCREEN_PROBE_SIZE as usize);
        let size = cols * rows;
        return Self{
            count: size as u32,
            data:  vec![GiGpuProbeData { camera_pose: Vec2::ZERO }; size],
        }
    }
}

#[derive(Clone, ShaderType, Default)]
pub struct GiGpuAmbientMaskData {
    pub(crate) center: Vec2,
    pub(crate) h_extent: Vec2,
}

impl GiGpuAmbientMaskData {
    pub fn new(center: Vec2, h_extent: Vec2) -> Self {
        Self { center, h_extent }
    }
}

#[derive(Clone, ShaderType, Default)]
pub struct GiGpuAmbientMaskBuffer {
    pub(crate) count: u32,
    #[size(runtime)]
    pub(crate) data: Vec<GiGpuAmbientMaskData>,
}