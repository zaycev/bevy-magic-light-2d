mod gi_pipeline_assets;
mod gi_gpu_types;
mod gi_pipeline;
mod gi_config;
pub mod gi_post_processing;
pub mod gi_component;
pub mod gi_floor_material;

use bevy::prelude::*;
use bevy::asset::load_internal_asset;
use bevy::sprite::Material2dPlugin;
use bevy::render::extract_resource::{ExtractResourcePlugin};
use bevy::render::render_graph::{self, RenderGraph};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext};
use bevy::render::{RenderApp, RenderStage};
use crate::gi::gi_post_processing::{
    PostProcessingTarget,
    PostProcessingMaterial,
    setup_post_processing_camera,
};
pub use crate::gi::gi_pipeline::{GiBlendTarget, GiTarget};

use self::gi_config::{
    SHADER_GI_CAMERA,
    SHADER_GI_TYPES,
    SHADER_GI_ATTENUATION,
    SHADER_GI_HALTON,
    SHADER_GI_MATH,
    GI_SCREEN_PROBE_SIZE,
};
use self::gi_pipeline::{
    GiPipeline,
    GiPipelineBindGroups,
    system_setup_gi_pipeline,
    system_queue_bind_groups,
};
use self::gi_pipeline_assets::{
    GiComputeAssets,
    system_extract_gi_assets,
    system_prepare_gi_assets
};

use crate::SCREEN_SIZE;
pub use crate::gi::gi_component::{
    LightSource,
    LightOccluder,
};
use crate::gi::gi_pipeline::GiPipelineTargetsWrapper;


const SIZE: (u32, u32) = (SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32);
const WORKGROUP_SIZE: u32 = 8;

/// Scaler for resolution of SDF texture.
///
pub enum SdfResolutionFactor {
    FULL,
    HALF,
    QUARTER,
}

pub struct GiComputePluginSettings {
    pub sdf_resolution_factor: SdfResolutionFactor,
}

pub struct GiComputePlugin;

impl Plugin for GiComputePlugin {
    fn build(&self, app: &mut App) {

        app.add_plugin(ExtractResourcePlugin::<GiPipelineTargetsWrapper>::default())
           .add_plugin(Material2dPlugin::<PostProcessingMaterial>::default())
           .init_resource::<PostProcessingTarget>()
           .init_resource::<GiPipelineTargetsWrapper>()
           .add_startup_system(system_setup_gi_pipeline)
           .add_startup_system(setup_post_processing_camera.after(system_setup_gi_pipeline));

        load_internal_asset!(
            app,
            SHADER_GI_CAMERA,
            "shaders/gi_camera.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_TYPES,
            "shaders/gi_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_ATTENUATION,
            "shaders/gi_attenuation.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_HALTON,
            "shaders/gi_halton.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_MATH,
            "shaders/gi_math.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GiPipeline>()
            .init_resource::<GiComputeAssets>()
            .add_system_to_stage(RenderStage::Extract, system_extract_gi_assets)
            .add_system_to_stage(RenderStage::Prepare, system_prepare_gi_assets)
            .add_system_to_stage(RenderStage::Queue,   system_queue_bind_groups);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("gi_compute", GiComputeNode::default());
        render_graph
            .add_node_edge(
                "gi_compute",
                bevy::render::main_graph::node::CAMERA_DRIVER,
            )
            .unwrap();
    }
}


struct GiComputeNode {}

impl Default for GiComputeNode {
    fn default() -> Self {
        Self {}
    }
}


impl render_graph::Node for GiComputeNode {
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        _:              &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world:          &World,
    ) -> Result<(), render_graph::NodeRunError> {

        if let Some(pipeline_bind_groups) = world.get_resource::<GiPipelineBindGroups>() {

            let     pipeline_cache = world.resource::<PipelineCache>();
            let     pipeline       = world.resource::<GiPipeline>();

            if let (
                Some(sdf_pipeline),
                Some(ss_probe_pipeline),
                Some(ss_bounce_pipeline),
                Some(ss_blend_pipeline),
                Some(ss_filter_pipeline)
            ) = (
                pipeline_cache.get_compute_pipeline(pipeline.sdf_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_probe_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_bounce_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_blend_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_filter_pipeline)
            ) {

                let mut pass = render_context.command_encoder.begin_compute_pass(&ComputePassDescriptor{
                    label: Some("gi_pass".into())
                });

                {
                    let grid_w = SIZE.0 / WORKGROUP_SIZE;
                    let grid_h = SIZE.1 / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.sdf_bind_group, &[]);
                    pass.set_pipeline(sdf_pipeline);
                    pass.dispatch_workgroups(
                        grid_w,
                        grid_h,
                        1
                    );
                }

                {
                    let workgroup_size = 8;
                    let grid_w = (SIZE.0 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    let grid_h = (SIZE.1 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_probe_bind_group, &[]);
                    pass.set_pipeline(ss_probe_pipeline);
                    pass.dispatch_workgroups(
                        grid_w,
                        grid_h,
                        1
                    );
                }

                {
                    let workgroup_size = 8;
                    let grid_w = (SIZE.0 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    let grid_h = (SIZE.1 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_bounce_bind_group, &[]);
                    pass.set_pipeline(ss_bounce_pipeline);
                    pass.dispatch_workgroups(
                        grid_w,
                        grid_h,
                        1
                    );
                }

                {
                    let workgroup_size = 8;
                    let grid_w = (SIZE.0 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    let grid_h = (SIZE.1 / GI_SCREEN_PROBE_SIZE as u32) / workgroup_size;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_blend_bind_group, &[]);
                    pass.set_pipeline(ss_blend_pipeline);
                    pass.dispatch_workgroups(
                        grid_w,
                        grid_h,
                        1
                    );
                }

                {
                    let grid_w = SIZE.0 / WORKGROUP_SIZE;
                    let grid_h = SIZE.1 / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_filter_bind_group, &[]);
                    pass.set_pipeline(ss_filter_pipeline);
                    pass.dispatch_workgroups(
                        grid_w,
                        grid_h,
                        1
                    );
                }
            }

        } else {
            log::warn!("Failed to get bind groups");
        }

        Ok(())
    }
}
