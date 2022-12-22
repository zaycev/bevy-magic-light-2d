use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::ImageSampler;

use super::gi_config::GI_SCREEN_PROBE_SIZE;
use super::gi_gpu_types::{
    GiGpuCameraParams,
    GiGpuLightOccluderBuffer,
    GiGpuLightSourceBuffer,
    GiGpuState, GiGpuProbeDataBuffer, GiGpuAmbientMaskBuffer,
};
use super::gi_pipeline_assets::GiComputeAssets;

const SDF_TARGET_FORMAT:       TextureFormat = TextureFormat::R16Float;
const SS_PROBE_TARGET_FORMAT:  TextureFormat = TextureFormat::Rgba16Float;
const SS_BOUNCE_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
const SS_BLEND_TARGET_FORMAT:  TextureFormat = TextureFormat::Rgba32Float;
const SS_FILTER_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

const SDF_PIPELINE_ENTRY:       &str = "main";
const SS_PROBE_PIPELINE_ENTRY:  &str = "main";
const SS_BOUNCE_PIPELINE_ENTRY: &str = "main";
const SS_BLEND_PIPELINE_ENTRY:  &str = "main";
const SS_FILTER_PIPELINE_ENTRY: &str = "main";

#[derive(Component)]
pub struct GiTarget;
#[derive(Component)]
pub struct GiSdfTarget;
#[derive(Component)]
pub struct SSProbeTarget;
#[derive(Component)]
pub struct GiBounceTarget;
#[derive(Component)]
pub struct GiBlendTarget;
#[derive(Component)]
pub struct GiFilterTarget;

#[allow(dead_code)]
#[derive(Clone, Resource, ExtractResource, Default)]
pub struct GiPipelineTargetsWrapper {
    pub(crate) targets: Option<GiPipelineTargets>,
}

#[derive(Clone)]
pub struct GiPipelineTargets {
    pub(crate) sdf_target:        Handle<Image>,
    pub(crate) ss_probe_target:   Handle<Image>,
    pub(crate) ss_bounce_target:  Handle<Image>,
    pub(crate) ss_blend_target:   Handle<Image>,
    pub(crate) ss_filter_target:  Handle<Image>,
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct GiPipelineBindGroups {
    pub(crate) sdf_bind_group:        BindGroup,
    pub(crate) ss_blend_bind_group:   BindGroup,
    pub(crate) ss_probe_bind_group:   BindGroup,
    pub(crate) ss_bounce_bind_group:  BindGroup,
    pub(crate) ss_filter_bind_group:  BindGroup,
}

fn create_texture_2d(size: (u32, u32), format: TextureFormat) -> Image {

    let mut image = Image::new_fill(Extent3d {
        width: size.0,
        height: size.1,
        ..Default::default()
    }, TextureDimension::D2, &[
        0, 0, 0, 0,  0, 0, 0, 0,
        0, 0, 0, 0,  0, 0, 0, 0,
        0, 0, 0, 0,  0, 0, 0, 0,
        0, 0, 0, 0,  0, 0, 0, 0,
    ], format);

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST |
        TextureUsages::STORAGE_BINDING |
        TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        address_mode_u: AddressMode::ClampToBorder,
        address_mode_v: AddressMode::ClampToBorder,
        address_mode_w: AddressMode::ClampToBorder,
        ..Default::default()
    });

    image
}

pub fn system_setup_gi_pipeline(
    mut commands:           Commands,
    mut images:             ResMut<Assets<Image>>,
    mut gi_compute_targets: ResMut<GiPipelineTargetsWrapper>,
    windows:                Res<Windows>,
) {
    let window = windows.get_primary().expect("failed to get window");
    let target_size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };

    let sdf_tex       = create_texture_2d((target_size.width, target_size.height), SDF_TARGET_FORMAT);
    let ss_probe_tex  = create_texture_2d((target_size.width, target_size.height), SS_PROBE_TARGET_FORMAT);
    let ss_bounce_tex = create_texture_2d((target_size.width, target_size.height), SS_BOUNCE_TARGET_FORMAT);
    let ss_blend_tex  = create_texture_2d((
        target_size.width  / (GI_SCREEN_PROBE_SIZE as u32),
        target_size.height / (GI_SCREEN_PROBE_SIZE as u32)
    ), SS_BLEND_TARGET_FORMAT);
    let ss_filter_tex = create_texture_2d((target_size.width, target_size.height), SS_FILTER_TARGET_FORMAT);

    let sdf_target       = images.add(sdf_tex);
    let ss_probe_target  = images.add(ss_probe_tex);
    let ss_bounce_target = images.add(ss_bounce_tex);
    let ss_blend_target  = images.add(ss_blend_tex);
    let ss_filter_target = images.add(ss_filter_tex);

    let sdf_image_entity = commands
        .spawn_empty()
        .insert(Name::new("sdf_target"))
        .insert(Sprite {
            custom_size: Some(Vec2::new(
                target_size.width as f32,
                target_size.height as f32,
            )),
            ..default()
        })
        .insert(sdf_target.clone())
        .insert(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(GiSdfTarget)
        .insert(GiTarget)
        .id();

    let ss_probe_image_entity = commands
        .spawn_empty()
        .insert(Name::new("ss_probe_target"))
        .insert(Sprite {
            custom_size: Some(Vec2::new(
                target_size.width as f32,
                target_size.height as f32,
            )),
            ..default()
        })
        .insert(ss_probe_target.clone())
        .insert(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(SSProbeTarget)
        .insert(GiTarget)
        .id();

    let ss_bounce_image_entity = commands
        .spawn_empty()
        .insert(Name::new("ss_bounce_target"))
        .insert(Sprite {
            custom_size: Some(Vec2::new(
                target_size.width as f32,
                target_size.height as f32,
            )),
            ..default()
        })
        .insert(ss_bounce_target.clone())
        .insert(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(GiBounceTarget)
        .insert(GiTarget)
        .id();

    let ss_blend_image_entity = commands
        .spawn_empty()
        .insert(Name::new("ss_blend_target"))
        .insert(Sprite {
            custom_size: Some(Vec2::new(
                target_size.width as f32,
                target_size.height as f32,
            )),
            ..default()
        })
        .insert(ss_blend_target.clone())
        .insert(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(GiBlendTarget)
        .insert(GiTarget)
        .id();

    let ss_filter_image_entity = commands
        .spawn_empty()
        .insert(Name::new("ss_filter_target"))
        .insert(Sprite {
            custom_size: Some(Vec2::new(
                target_size.width as f32,
                target_size.height as f32,
            )),
            ..default()
        })
        .insert(ss_filter_target.clone())
        .insert(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(GiFilterTarget)
        .insert(GiTarget)
        .id();

    commands
        .spawn_empty()
        .insert(Name::new("gi_pipeline"))
        .insert(SpatialBundle::default())
        .add_child(sdf_image_entity)
        .add_child(ss_probe_image_entity)
        .add_child(ss_bounce_image_entity)
        .add_child(ss_blend_image_entity)
        .add_child(ss_filter_image_entity);

    commands.spawn(Camera2dBundle::default());

    gi_compute_targets.targets = Some(GiPipelineTargets {
        sdf_target,
        ss_probe_target,
        ss_bounce_target,
        ss_blend_target,
        ss_filter_target,
    });
}

#[derive(Resource)]
pub struct GiPipeline {
    pub sdf_bind_group_layout:        BindGroupLayout,
    pub sdf_pipeline:                 CachedComputePipelineId,
    pub ss_probe_bind_group_layout:   BindGroupLayout,
    pub ss_probe_pipeline:            CachedComputePipelineId,
    pub ss_bounce_bind_group_layout:  BindGroupLayout,
    pub ss_bounce_pipeline:           CachedComputePipelineId,
    pub ss_blend_bind_group_layout:   BindGroupLayout,
    pub ss_blend_pipeline:            CachedComputePipelineId,
    pub ss_filter_bind_group_layout:  BindGroupLayout,
    pub ss_filter_pipeline:           CachedComputePipelineId,
}

pub fn system_queue_bind_groups(
    mut commands:       Commands,
    pipeline:           Res<GiPipeline>,
    gpu_images:         Res<RenderAssets<Image>>,
    targets_wrapper:    Res<GiPipelineTargetsWrapper>,
    gi_compute_assets:  Res<GiComputeAssets>,
    render_device:      Res<RenderDevice>,
) {
    if let (
            Some(light_sources),
            Some(light_occluders),
            Some(camera_params),
            Some(gi_state),
            Some(probes),
            Some(ambient_masks),
        ) = (
            gi_compute_assets.light_sources.binding(),
            gi_compute_assets.light_occluders.binding(),
            gi_compute_assets.camera_params.binding(),
            gi_compute_assets.gi_state.binding(),
            gi_compute_assets.probes.binding(),
            gi_compute_assets.ambient_masks.binding(),
    ) {

        let targets = targets_wrapper.targets.as_ref().expect("Targets should be initialized");

        let sdf_view_image  = &gpu_images[&targets.sdf_target];
        let ss_probe_image  = &gpu_images[&targets.ss_probe_target];
        let ss_bounce_image = &gpu_images[&targets.ss_bounce_target];
        let ss_blend_image  = &gpu_images[&targets.ss_blend_target];
        let ss_filter_image = &gpu_images[&targets.ss_filter_target];

        let sdf_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "gi_sdf_bind_group".into(),
            layout: &pipeline.sdf_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_occluders.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&sdf_view_image.texture_view),
                },
            ],
        });

        let ss_probe_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "gi_ss_probe_bind_group".into(),
            layout: &pipeline.ss_probe_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: gi_state.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: probes.clone(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: ambient_masks.clone(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: light_sources.clone(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&sdf_view_image.texture_view),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::TextureView(&ss_probe_image.texture_view),
                },
            ],
        });

        let ss_bounce_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "gi_bounce_bind_group".into(),
            layout: &pipeline.ss_bounce_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: gi_state.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&sdf_view_image.texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&ss_probe_image.texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&ss_bounce_image.texture_view),
                },
            ],
        });

        let ss_blend_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "gi_blend_bind_group".into(),
            layout: &pipeline.ss_blend_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: gi_state.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: probes.clone(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&sdf_view_image.texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&ss_bounce_image.texture_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&ss_blend_image.texture_view),
                },
            ],
        });

        let ss_filter_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "ss_filter_bind_group".into(),
            layout: &pipeline.ss_filter_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: gi_state.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: probes.clone(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&sdf_view_image.texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&ss_blend_image.texture_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&ss_filter_image.texture_view),
                },
            ],
        });

        commands.insert_resource(GiPipelineBindGroups {
            sdf_bind_group,
            ss_probe_bind_group,
            ss_bounce_bind_group,
            ss_blend_bind_group,
            ss_filter_bind_group,
        });
    }
}

impl FromWorld for GiPipeline {
    fn from_world(world: &mut World) -> Self {

        let render_device = world.resource::<RenderDevice>();

        let sdf_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sdf_bind_group_layout"),
            entries: &[
                // Camera.
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuCameraParams::min_size()),
                    },
                    count: None,
                },
                // Light occluders.
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuLightOccluderBuffer::min_size()),
                    },
                    count: None,
                },
                // SDF texture.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let ss_probe_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ss_probe_bind_group_layout"),
            entries: &[
                // Camera.
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuCameraParams::min_size()),
                    },
                    count: None,
                },

                // GI State.
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuState::min_size()),
                    },
                    count: None,
                },

                // Probes.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuProbeDataBuffer::min_size()),
                    },
                    count: None,
                },

                // AmbientMasks.
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuAmbientMaskBuffer::min_size()),
                    },
                    count: None,
                },

                // Light sources.
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuLightSourceBuffer::min_size()),
                    },
                    count: None,
                },

                // SDF.
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Probe.
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: SS_PROBE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let ss_bounce_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ss_bounce_bind_group_layout".into()),
            entries: &[
                // Camera.
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuCameraParams::min_size()),
                    },
                    count: None,
                },

                // GI State.
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuState::min_size()),
                    },
                    count: None,
                },

                // SDF.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Probe.
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SS_PROBE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Bounce.
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: SS_BOUNCE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });


        let ss_blend_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ss_blend_bind_group_layout".into()),
            entries: &[
                // Camera.
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuCameraParams::min_size()),
                    },
                    count: None,
                },

                // GI State.
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuState::min_size()),
                    },
                    count: None,
                },

                // Probes.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuProbeDataBuffer::min_size()),
                    },
                    count: None,
                },

                // SDF.
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Bounces.
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SS_BOUNCE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Blend.
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: SS_BLEND_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });


        let ss_filter_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ss_filter_bind_group_layout".into()),
            entries: &[
                // Camera.
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuCameraParams::min_size()),
                    },
                    count: None,
                },

                // GI State.
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuState::min_size()),
                    },
                    count: None,
                },

                // Probes.
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GiGpuProbeDataBuffer::min_size()),
                    },
                    count: None,
                },

                // SDF.
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Blend.
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: SS_BLEND_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // SS Filter.
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: SS_FILTER_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let (
            shader_sdf,
            gi_ss_probe,
            gi_ss_bounce,
            gi_ss_blend,
            gi_ss_filter,
        ) = {
            let assets_server = world.resource::<AssetServer>();
            (
                assets_server.load("shaders/gi_sdf.wgsl"),
                assets_server.load("shaders/gi_ss_probe.wgsl"),
                assets_server.load("shaders/gi_ss_bounce.wgsl"),
                assets_server.load("shaders/gi_ss_blend.wgsl"),
                assets_server.load("shaders/gi_ss_filter.wgsl"),
            )
        };

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();


        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gi_sdf_pipeline".into()),
            layout: Some(vec![sdf_bind_group_layout.clone()]),
            shader: shader_sdf,
            shader_defs: vec![],
            entry_point: SDF_PIPELINE_ENTRY.into(),
        });

        let ss_probe_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gi_ss_probe_pipeline".into()),
            layout: Some(vec![ss_probe_bind_group_layout.clone()]),
            shader: gi_ss_probe,
            shader_defs: vec![],
            entry_point: SS_PROBE_PIPELINE_ENTRY.into(),
        });

        let ss_bounce_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gi_ss_bounce_pipeline".into()),
            layout: Some(vec![ss_bounce_bind_group_layout.clone()]),
            shader: gi_ss_bounce,
            shader_defs: vec![],
            entry_point: SS_BOUNCE_PIPELINE_ENTRY.into(),
        });

        let ss_blend_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gi_blend_pipeline".into()),
            layout: Some(vec![ss_blend_bind_group_layout.clone()]),
            shader: gi_ss_blend,
            shader_defs: vec![],
            entry_point: SS_BLEND_PIPELINE_ENTRY.into(),
        });

        let ss_filter_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("gi_filer_pipeline".into()),
            layout: Some(vec![ss_filter_bind_group_layout.clone()]),
            shader: gi_ss_filter,
            shader_defs: vec![],
            entry_point: SS_FILTER_PIPELINE_ENTRY.into(),
        });

        GiPipeline {
            //
            sdf_bind_group_layout,
            sdf_pipeline,
            //
            ss_probe_bind_group_layout,
            ss_probe_pipeline,
            //
            ss_bounce_bind_group_layout,
            ss_bounce_pipeline,
            //
            ss_blend_bind_group_layout,
            ss_blend_pipeline,
            //
            ss_filter_bind_group_layout,
            ss_filter_pipeline,
        }
    }
}
