use bevy::prelude::*;
use bevy::render::render_resource::{StorageBuffer, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::Extract;
use rand::{Rng, thread_rng};

use super::gi_config::GI_SCREEN_PROBE_SIZE;
use super::gi_gpu_types::GiGpuState;

use crate::{MainCamera, SCREEN_SIZE};
use crate::gi::gi_component::{GiAmbientLight, LightSource, LightOccluder, AmbientMask};
use crate::gi::gi_gpu_types::{
    GiGpuCameraParams,
    GiGpuLightSource,
    GiGpuLightSourceBuffer,
    GiGpuLightOccluder,
    GiGpuLightOccluderBuffer,
    GiGpuProbeDataBuffer,
    GiGpuAmbientMaskBuffer,
    GiGpuAmbientMaskData,
};

#[derive(Default, Resource)]
pub struct GiComputeAssets {
    pub(crate) light_sources:   StorageBuffer<GiGpuLightSourceBuffer>,
    pub(crate) light_occluders: StorageBuffer<GiGpuLightOccluderBuffer>,
    pub(crate) camera_params:   UniformBuffer<GiGpuCameraParams>,
    pub(crate) gi_state:        UniformBuffer<GiGpuState>,
    pub(crate) probes:          StorageBuffer<GiGpuProbeDataBuffer>,
    pub(crate) ambient_masks:   StorageBuffer<GiGpuAmbientMaskBuffer>,
}

impl GiComputeAssets {
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.light_sources.write_buffer(device, queue);
        self.light_occluders.write_buffer(device, queue);
        self.camera_params.write_buffer(device, queue);
        self.gi_state.write_buffer(device, queue);
        self.probes.write_buffer(device, queue);
        self.ambient_masks.write_buffer(device, queue);
    }
}

pub(crate) fn system_prepare_gi_assets(
        render_device:     Res<RenderDevice>,
        render_queue:      Res<RenderQueue>,
    mut gi_compute_assets: ResMut<GiComputeAssets>,
) {
    gi_compute_assets.write_buffer(&render_device, &render_queue);
}

pub(crate) fn system_extract_gi_assets(
    mut gi_compute_assets:   ResMut<GiComputeAssets>,
    mut frame_counter:       Local<i32>,
        query_lights:        Extract<Query<(&Transform, &LightSource, &ComputedVisibility)>>,
        query_occluders:     Extract<Query<(&LightOccluder, &Transform, &ComputedVisibility)>>,
        query_camera:        Extract<Query<(&Camera, &GlobalTransform), With<MainCamera>>>,
        query_masks:         Extract<Query<(&Transform, &AmbientMask)>>,
        query_ambient_light: Extract<Query<&GiAmbientLight>>,
) {

    {
        let mut light_sources  = gi_compute_assets.light_sources.get_mut();
        let mut rng            = thread_rng();
        light_sources.count = 0;
        light_sources.data.clear();
        for (transform, light_source, visibility) in query_lights.iter() {
            if visibility.is_visible() {
                light_sources.count += 1;
                light_sources.data.push(GiGpuLightSource::new(
                    LightSource{
                        intensity: light_source.intensity + rng.gen_range(-1.0..1.0) * light_source.jitter_intensity,
                        ..*light_source
                    },
                    Vec2::new(
                        transform.translation.x + rng.gen_range(-1.0..1.0) * light_source.jitter_translation,
                        transform.translation.y + rng.gen_range(-1.0..1.0) * light_source.jitter_translation,
                    ),
                ));
            }
        }
    }

    {
        let mut light_occluders = gi_compute_assets.light_occluders.get_mut();
        light_occluders.count = 0;
        light_occluders.data.clear();
        for (occluder, transform, visibility) in query_occluders.iter() {
            if visibility.is_visible() {
                light_occluders.count += 1;
                light_occluders.data.push(GiGpuLightOccluder::new(
                    transform.translation.truncate(),
                    occluder.h_size,
                ));
            }
        }
    }

    {
        let mut ambient_masks = gi_compute_assets.ambient_masks.get_mut();
        ambient_masks.count = 0;
        ambient_masks.data.clear();
        for (transform, mask) in query_masks.iter() {
            ambient_masks.count += 1;
            ambient_masks.data.push(GiGpuAmbientMaskData::new(
                transform.translation.truncate(),
                mask.h_size,
            ));
        }
    }

    {
        if let Ok((camera, camera_global_transform)) = query_camera.get_single() {
            let mut camera_params = gi_compute_assets.camera_params.get_mut();
            let projection         = camera.projection_matrix();
            let inverse_projection = projection.inverse();
            let view               = camera_global_transform.compute_matrix();
            let inverse_view       = view.inverse();

            camera_params.view_proj         = projection * inverse_view;
            camera_params.inverse_view_proj = view * inverse_projection;
            camera_params.screen_size       = Vec2::new(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32);
            camera_params.screen_size_inv   = Vec2::new(1.0 / SCREEN_SIZE.0 as f32, 1.0 / SCREEN_SIZE.1 as f32);

            let probes = gi_compute_assets.probes.get_mut();
            probes.data[*frame_counter as usize].camera_pose = camera_global_transform.translation().truncate();
        } else {
            let probes = gi_compute_assets.probes.get_mut();
            probes.data[*frame_counter as usize].camera_pose = Vec2::ZERO;
        }
    }

    {
        let cols = SCREEN_SIZE.0 as i32 / GI_SCREEN_PROBE_SIZE;
        let rows = SCREEN_SIZE.1 as i32 / GI_SCREEN_PROBE_SIZE;
        let mut gi_state = gi_compute_assets.gi_state.get_mut();
        gi_state.gi_frame_counter  = *frame_counter;
        gi_state.ss_probe_size     = GI_SCREEN_PROBE_SIZE;
        gi_state.ss_atlas_cols     = cols;
        gi_state.ss_atlas_rows     = rows;
    }

    {
        let mut gi_state = gi_compute_assets.gi_state.get_mut();
        gi_state.gi_ambient = Vec3::splat(0.0);
        for new_gi_state in query_ambient_light.iter(){
            gi_state.gi_ambient.x = new_gi_state.color.r() * new_gi_state.intensity;
            gi_state.gi_ambient.y = new_gi_state.color.g() * new_gi_state.intensity;
            gi_state.gi_ambient.z = new_gi_state.color.b() * new_gi_state.intensity;
        }
    }

    *frame_counter = (*frame_counter + 1) % (GI_SCREEN_PROBE_SIZE * GI_SCREEN_PROBE_SIZE);
}