use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::graph::CameraDriverLabel;
use bevy::render::render_graph::{
    Node,
    NodeRunError,
    RenderGraph,
    RenderGraphApp,
    RenderGraphContext,
    RenderLabel,
    ViewNode,
    ViewNodeRunner,
};
use bevy::render::render_resource::{
    BindGroupEntries,
    Operations,
    PipelineCache,
    RenderPassColorAttachment,
    RenderPassDescriptor,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::view::ViewTarget;
use bevy::render::{render_graph, Extract, Render, RenderApp};
use bevy::window::PrimaryWindow;

use crate::components::LightOmni;
use crate::hud::{hud_setup, hud_update};
use crate::pipelines::pipeline_basic::{
    MagicLight2DPipelineBasicAssets,
    MagicLight2DPipelineBasicImpl,
};
use crate::pipelines::Magic2DPipelineParams;

#[derive(Clone, Resource)]
pub struct Magic2DPluginConfig
{
    pub pipeline: Magic2DPipelineParams,
}

pub struct Magic2DPlugin
{
    pub config: Magic2DPluginConfig,
}

impl Plugin for Magic2DPlugin
{
    fn build(&self, app: &mut App)
    {
        // Initialize plugin.
        app.insert_resource(self.config.clone());
        app.add_systems(Startup, configure_magic_light);
        app.add_systems(Update, configure_magic_light);

        // Initialize debug hud.
        if cfg!(feature = "debug_hud") {
            app.add_systems(Startup, hud_setup);
            app.add_systems(Update, hud_update);
        }

        // Setup extract, prepare, and queue systems.
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, pipeline_extract);
        render_app.add_systems(Render, pipeline_prepare);
        render_app.add_systems(Render, pipeline_queue);

        // Add render graph nodes.
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        // Compute nodes.
        render_graph.add_node(MagicLightComputeNodeLabel, MagicLightComputeNode::default());
        render_graph.add_node_edge(MagicLightComputeNodeLabel, CameraDriverLabel);

        // Post-processing nodes.
        let label = MagicLightPostProcessingNodeLabel;
        let edges = (
            Node2d::Tonemapping,
            label,
            Node2d::EndMainPassPostProcessing,
        );
        render_app
            .add_render_graph_node::<ViewNodeRunner<MagicLightPostProcessingNode>>(Core2d, label)
            .add_render_graph_edges(Core2d, edges);
    }

    fn finish(&self, app: &mut App)
    {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // Initialize the pipelines.
        render_app
            .init_resource::<MagicLight2DPipelineBasicAssets>()
            .init_resource::<MagicLight2DPipelineBasicImpl>();
    }
}

pub fn pipeline_extract(
    queries: (
        Extract<Query<&Window, With<PrimaryWindow>>>,
        Extract<Query<(&GlobalTransform, &Camera)>>,
        Extract<Query<(&GlobalTransform, &LightOmni, &InheritedVisibility)>>,
    ),

    mut gpu_assets_basic: ResMut<MagicLight2DPipelineBasicAssets>,
)
{
    let (query_window, query_camera, query_light) = queries;
    let Ok((transform, camera)) = query_camera.get_single() else {
        return;
    };
    let Ok(window) = query_window.get_single() else {
        return;
    };

    {
        gpu_assets_basic
            .globals
            .get_mut()
            .set_camera_params(window, camera, transform);
    }

    {
        let buf_omni_lights = gpu_assets_basic.lights_omni.get_mut();
        buf_omni_lights.count = 0;
        buf_omni_lights.data.clear();
        for (transform, light, hvis) in query_light.iter() {
            if !hvis.get() {
                continue;
            }

            // TODO: visibility check
            buf_omni_lights.count += 1;
            buf_omni_lights.data.push(light.as_gpu(transform));
        }
    }
}

pub fn pipeline_prepare(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut gpu_assets_basic: ResMut<MagicLight2DPipelineBasicAssets>,
)
{
    gpu_assets_basic.write_buffer(&render_device, &render_queue);
}

pub fn pipeline_queue() {}

/// NodeLabel for doing light computations.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MagicLightComputeNodeLabel;

/// Node for doing light computations.
#[derive(Default)]
struct MagicLightComputeNode;

impl Node for MagicLightComputeNode
{
    fn update(&mut self, _world: &mut World) {}
    fn run(
        &self,
        _render_graph: &mut render_graph::RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), render_graph::NodeRunError>
    {
        Ok(())
    }
}

/// NodeLabel for doing light computations.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, RenderLabel)]
pub struct MagicLightPostProcessingNodeLabel;

/// Node for doing light computations.
#[derive(Default)]
struct MagicLightPostProcessingNode;

impl ViewNode for MagicLightPostProcessingNode
{
    type ViewQuery = (&'static ViewTarget,);

    fn run(
        &self,
        _render_graph_graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target,): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError>
    {
        let basic_pipeline = world.resource::<MagicLight2DPipelineBasicImpl>();
        let pipeline_cache = world.resource::<PipelineCache>();
        if let Some(pipeline) = pipeline_cache.get_render_pipeline(basic_pipeline.pipeline_id) {
            let assets = world.resource::<MagicLight2DPipelineBasicAssets>();
            let binding_globals = assets
                .globals
                .binding()
                .expect("Missing global params buffer");
            let binding_lights_omni = assets
                .lights_omni
                .binding()
                .expect("Missing lights omni buffer");
            let target = view_target.post_process_write();
            let bind_group = render_context.render_device().create_bind_group(
                "magic_light_2d_post_processing_bind_group",
                &basic_pipeline.layout,
                &BindGroupEntries::sequential((
                    target.source,
                    &basic_pipeline.sampler,
                    binding_globals.clone(),
                    binding_lights_omni.clone(),
                )),
            );

            // Begin render pass.
            let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label:                    Some("magic_light_2d_render_pass"),
                color_attachments:        &[Some(RenderPassColorAttachment {
                    view:           target.destination,
                    resolve_target: None,
                    ops:            Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes:         None,
                occlusion_query_set:      None,
            });

            pass.set_render_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

pub fn configure_magic_light(mut _cmds: Commands, _plugin_config: Res<Magic2DPluginConfig>) {}
