use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::{self, RenderGraph};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderContext;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::sprite::Material2dPlugin;
use bevy::window::PrimaryWindow;

use crate::gi::compositing::{
    setup_post_processing_camera, PostProcessingMaterial, PostProcessingTarget,
};
use crate::gi::constants::*;
use crate::gi::pipeline::{
    system_queue_bind_groups, system_setup_gi_pipeline, LightPassPipeline,
    LightPassPipelineBindGroups, PipelineTargetsWrapper,
};
use crate::gi::pipeline_assets::{
    system_extract_pipeline_assets, system_prepare_pipeline_assets, LightPassPipelineAssets,
};
use crate::gi::resource::ComputedTargetSizes;
use crate::prelude::BevyMagicLight2DSettings;

mod constants;
mod pipeline;
mod pipeline_assets;
mod types_gpu;

pub mod compositing;
pub mod render_layer;
pub mod resource;
pub mod types;

const WORKGROUP_SIZE: u32 = 8;

pub struct BevyMagicLight2DPlugin;

impl Plugin for BevyMagicLight2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<PipelineTargetsWrapper>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
        ))
        .init_resource::<PostProcessingTarget>()
        .init_resource::<PipelineTargetsWrapper>()
        .init_resource::<BevyMagicLight2DSettings>()
        .init_resource::<ComputedTargetSizes>()
        .add_systems(
            PreStartup,
            (
                detect_target_sizes,
                system_setup_gi_pipeline.after(detect_target_sizes),
                setup_post_processing_camera.after(system_setup_gi_pipeline),
            )
                .chain(),
        );

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

        load_internal_asset!(
            app,
            SHADER_GI_RAYMARCH,
            "shaders/gi_raymarch.wgsl",
            Shader::from_wgsl
        );
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(ExtractSchedule, system_extract_pipeline_assets)
            .add_systems(
                Render,
                (
                    system_prepare_pipeline_assets.in_set(RenderSet::Prepare),
                    system_queue_bind_groups.in_set(RenderSet::Queue),
                ),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_pass_2d", LightPass2DNode::default());
        render_graph.add_node_edge(
            "light_pass_2d",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        )
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<LightPassPipeline>()
            .init_resource::<LightPassPipelineAssets>()
            .init_resource::<ComputedTargetSizes>();
    }
}

#[derive(Default)]
struct LightPass2DNode {}

#[rustfmt::skip]
pub(crate) fn detect_target_sizes(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut target_sizes: ResMut<ComputedTargetSizes>)
{
    let window = windows.get_single().expect("No primary window");
    let primary_size = Vec2::new(
        (window.physical_width() as f64 / window.scale_factor()) as f32,
        (window.physical_height() as f64 / window.scale_factor()) as f32,
    );

    target_sizes.primary_target_size = primary_size;
    target_sizes.primary_target_isize = target_sizes.primary_target_size.as_ivec2();
    target_sizes.primary_target_usize = target_sizes.primary_target_size.as_uvec2();

    target_sizes.sdf_target_size = primary_size * 0.5;
    target_sizes.sdf_target_isize = target_sizes.sdf_target_size.as_ivec2();
    target_sizes.sdf_target_usize = target_sizes.sdf_target_size.as_uvec2();
}

impl render_graph::Node for LightPass2DNode {
    fn update(&mut self, _world: &mut World) {}

    #[rustfmt::skip]
    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(pipeline_bind_groups) = world.get_resource::<LightPassPipelineBindGroups>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<LightPassPipeline>();
            let target_sizes = world.resource::<ComputedTargetSizes>();

            if let (
                Some(sdf_pipeline),
                Some(ss_probe_pipeline),
                Some(ss_bounce_pipeline),
                Some(ss_blend_pipeline),
                Some(ss_filter_pipeline),
            ) = (
                pipeline_cache.get_compute_pipeline(pipeline.sdf_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_probe_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_bounce_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_blend_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_filter_pipeline),
            ) {
                let primary_w = target_sizes.primary_target_usize.x;
                let primary_h = target_sizes.primary_target_usize.y;
                let sdf_w = target_sizes.sdf_target_usize.x;
                let sdf_h = target_sizes.sdf_target_usize.y;

                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("light_pass_2d"),
                        });

                {
                    let grid_w = sdf_w / WORKGROUP_SIZE;
                    let grid_h = sdf_h / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.sdf_bind_group, &[]);
                    pass.set_pipeline(sdf_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = (primary_w / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    let grid_h = (primary_h / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_probe_bind_group, &[]);
                    pass.set_pipeline(ss_probe_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = (primary_w / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    let grid_h = (primary_h / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_bounce_bind_group, &[]);
                    pass.set_pipeline(ss_bounce_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = (primary_w / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    let grid_h = (primary_h / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_blend_bind_group, &[]);
                    pass.set_pipeline(ss_blend_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = primary_w / WORKGROUP_SIZE;
                    let grid_h = primary_h / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_filter_bind_group, &[]);
                    pass.set_pipeline(ss_filter_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }
            }
        } else {
            log::warn!("Failed to get bind groups");
        }

        Ok(())
    }
}
