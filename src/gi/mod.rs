use crate::{FloorCamera, ObjectsCamera, SpriteCamera, WallsCamera};
use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::{self, RenderGraph};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderContext;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::sprite::{Material2dPlugin, MaterialMesh2dBundle};
use bevy::window::{PrimaryWindow, WindowResized};

use self::pipeline::GiTargets;
use crate::gi::compositing::{
    setup_post_processing_camera, CameraTargets, PostProcessingMaterial, PostProcessingQuad,
};
use crate::gi::constants::*;
use crate::gi::pipeline::{
    system_queue_bind_groups, system_setup_gi_pipeline, GiTargetsWrapper, LightPassPipeline,
    LightPassPipelineBindGroups,
};
use crate::gi::pipeline_assets::{
    system_extract_pipeline_assets, system_prepare_pipeline_assets, LightPassPipelineAssets,
};
use crate::gi::resource::ComputedTargetSizes;
use crate::gi::util::AssetUtil;
use crate::prelude::BevyMagicLight2DSettings;

mod constants;
mod pipeline;
mod pipeline_assets;
mod types_gpu;

pub mod compositing;
pub mod render_layer;
pub mod resource;
pub mod types;
pub mod util;

const WORKGROUP_SIZE: u32 = 8;

pub struct BevyMagicLight2DPlugin;

impl Plugin for BevyMagicLight2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<GiTargetsWrapper>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
        ))
        .init_resource::<CameraTargets>()
        .init_resource::<GiTargetsWrapper>()
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
        )
        .add_systems(PreUpdate, recreate_targets_on_window_resize);

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

pub fn recreate_targets_on_window_resize(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PostProcessingMaterial>>,

    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_cameras: Query<(Entity, &mut Camera), With<SpriteCamera>>,

    query_floor_camera: Query<Entity, With<FloorCamera>>,
    query_walls_camera: Query<Entity, With<WallsCamera>>,
    query_objects_camera: Query<Entity, With<ObjectsCamera>>,
    query_post_processing_quad: Query<Entity, With<PostProcessingQuad>>,

    mut window_resized_evr: EventReader<WindowResized>,
    mut res_target_sizes: ResMut<ComputedTargetSizes>,
    res_plugin_config: Res<BevyMagicLight2DSettings>,

    mut images: ResMut<Assets<Image>>,

    mut gi_targets_wrapper: ResMut<GiTargetsWrapper>,
    mut camera_targets: ResMut<CameraTargets>,
) {
    for _ in window_resized_evr.iter() {
        let window = query_window
            .get_single()
            .expect("Expected exactly one primary window");

        *res_target_sizes =
            ComputedTargetSizes::from_window(window, &res_plugin_config.target_scaling_params);

        let quad_handle = meshes.set(
            AssetUtil::mesh("pp"),
            Mesh::from(shape::Quad::new(Vec2::new(
                res_target_sizes.primary_target_size.x,
                res_target_sizes.primary_target_size.y,
            ))),
        );

        let material_handle = materials.set(
            AssetUtil::material("pp"),
            PostProcessingMaterial::create(&camera_targets, &gi_targets_wrapper),
        );

        let new_gi_targets = GiTargets::create(&mut images, &res_target_sizes);
        let new_camera_targets = CameraTargets::create(&mut images, &res_target_sizes);

        // Recreate post-processing material.
        let post_processing_quad = query_post_processing_quad
            .get_single()
            .expect("Expected exactly one post-processing quad");
        commands
            .entity(post_processing_quad)
            .insert(MaterialMesh2dBundle {
                mesh: quad_handle.into(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    ..default()
                },
                ..default()
            });

        // Update cameras.

        for (camera_entity, mut camera) in &mut query_cameras {

            // if let Ok(_) = query_floor_camera.get(camera_entity)   { camera.target = RenderTarget::Image(new_gi_targets.) }
            // if let Ok(_) = query_walls_camera.get(camera_entity)   { camera.target = }
            // if let Ok(_) = query_objects_camera.get(camera_entity) { }
        }

        gi_targets_wrapper.targets = Some(new_gi_targets);
        *camera_targets = new_camera_targets;
    }
}

#[rustfmt::skip]
pub(crate) fn detect_target_sizes(
        query_window:      Query<&Window, With<PrimaryWindow>>,

        res_plugin_config: Res<BevyMagicLight2DSettings>,
    mut res_target_sizes:  ResMut<ComputedTargetSizes>,
)
{
    let window = query_window.get_single().expect("Expected exactly one primary window");
    *res_target_sizes = ComputedTargetSizes::from_window(window, &res_plugin_config.target_scaling_params);
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
