use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::{
    sampler,
    storage_buffer_read_only,
    texture_2d,
    uniform_buffer,
};
use bevy::render::render_resource::{
    BindGroupLayout,
    BindGroupLayoutEntries,
    CachedRenderPipelineId,
    ColorTargetState,
    ColorWrites,
    FragmentState,
    MultisampleState,
    PipelineCache,
    PrimitiveState,
    RenderPipelineDescriptor,
    Sampler,
    SamplerBindingType,
    SamplerDescriptor,
    ShaderStages,
    StorageBuffer,
    TextureFormat,
    TextureSampleType,
    UniformBuffer,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::gpu_types::{GpuGlobalParams, GpuLightOmniBuffer};

pub struct MagicLight2DPipelineBasic {}

/// The actual implementation of the basic pipeline as post processing step adding
/// light to the scene.
#[derive(Resource)]
pub struct MagicLight2DPipelineBasicImpl
{
    pub layout:      BindGroupLayout,
    pub sampler:     Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for MagicLight2DPipelineBasicImpl
{
    fn from_world(world: &mut World) -> Self
    {
        let device = world.resource::<RenderDevice>();
        let layout = device.create_bind_group_layout(
            "magic_2d_basic_post_processing_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::FRAGMENT,
                (
                    // The screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // The sampler that will be used to sample the screen texture
                    sampler(SamplerBindingType::Filtering),
                    // Global params.
                    uniform_buffer::<GpuGlobalParams>(false),
                    // Omni lights buffer.
                    storage_buffer_read_only::<GpuLightOmniBuffer>(false),
                ),
            ),
        );
        let sampler = device.create_sampler(&SamplerDescriptor::default());
        let shader = world.load_asset("shaders/basic/pass_post_processing.wgsl");
        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label:                Some("magic_2d_basic_post_processing".into()),
                    layout:               vec![layout.clone()],
                    vertex:               fullscreen_shader_vertex_state(),
                    fragment:             Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format:     TextureFormat::Rgba16Float,
                            blend:      None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive:            PrimitiveState::default(),
                    depth_stencil:        None,
                    multisample:          MultisampleState::default(),
                    push_constant_ranges: vec![],
                });
        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

/// Data that has to be passed into the pipeline to render the light.
#[derive(Default, Resource)]
pub struct MagicLight2DPipelineBasicAssets
{
    pub globals:     UniformBuffer<GpuGlobalParams>,
    pub lights_omni: StorageBuffer<GpuLightOmniBuffer>,
}

impl MagicLight2DPipelineBasicAssets
{
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue)
    {
        self.globals.write_buffer(device, queue);
        self.lights_omni.write_buffer(device, queue);
    }
}
