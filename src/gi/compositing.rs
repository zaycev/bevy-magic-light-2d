use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::{MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS};
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderDefVal, ShaderRef,
    SpecializedMeshPipelineError, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dKey, MaterialMesh2dBundle};

use crate::gi::pipeline::PipelineTargetsWrapper;
use crate::gi::resource::ComputedTargetSizes;

#[rustfmt::skip]
#[derive(AsBindGroup, TypeUuid, Clone, TypePath)]
#[uuid = "bc2f08eb-a0fb-43f1-a908-54871ea597d5"]
pub struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    floor_image:       Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    walls_image:       Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    objects_image:     Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    irradiance_image:  Handle<Image>,
}

#[derive(Resource, Default)]
pub struct PostProcessingTarget {
    pub handles: Option<(
        Handle<Image>, // Floor  layer.
        Handle<Image>, // Walls  layer.
        Handle<Image>, // Objects layer.
    )>,
}

impl Material2d for PostProcessingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gi_post_processing.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let shader_defs = &mut descriptor
            .fragment
            .as_mut()
            .expect("Fragment shader empty")
            .shader_defs;
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_DIRECTIONAL_LIGHTS".to_string(),
            MAX_DIRECTIONAL_LIGHTS as u32,
        ));
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_CASCADES_PER_LIGHT".to_string(),
            MAX_CASCADES_PER_LIGHT as u32,
        ));
        Ok(())
    }
}

#[rustfmt::skip]
pub fn setup_post_processing_camera(
    mut commands:                  Commands,
    mut meshes:                    ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
    mut images:                    ResMut<Assets<Image>>,
    mut post_processing_target:    ResMut<PostProcessingTarget>,

    gpu_targets_sizes:             Res<ComputedTargetSizes>,
    gpu_targets_wrapper:           Res<PipelineTargetsWrapper>,
) {
    let window_size = Extent3d {
        width:  gpu_targets_sizes.primary_target_usize.x,
        height: gpu_targets_sizes.primary_target_usize.y,
        ..default()
    };

    let mut floor_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("target_floor"),
            size: window_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                 | TextureUsages::COPY_DST
                 | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    let mut walls_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("target_walls"),
            size: window_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                 | TextureUsages::COPY_DST
                 | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    let mut objects_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("target_objects"),
            size: window_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                 | TextureUsages::COPY_DST
                 | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };





    // Fill images data with zeroes.
    floor_image.resize(window_size);
    walls_image.resize(window_size);
    objects_image.resize(window_size);

    // Create handles.
    let floor_image_handle   = images.add(floor_image);
    let walls_image_handle   = images.add(walls_image);
    let objects_image_handle = images.add(objects_image);

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        gpu_targets_sizes.primary_target_size.x,
        gpu_targets_sizes.primary_target_size.y,
    ))));

    // This material has the texture that has been rendered.
    post_processing_target.handles = Some((
        floor_image_handle.clone(),
        walls_image_handle.clone(),
        objects_image_handle.clone(),
    ));

    let material_handle = post_processing_materials.add(PostProcessingMaterial {

        floor_image:  floor_image_handle,
        walls_image:  walls_image_handle,
        objects_image: objects_image_handle,

        irradiance_image: gpu_targets_wrapper
            .targets
            .as_ref()
            .expect("Targets must be initialized")
            .ss_filter_target
            .clone(),

    });

    // This specifies the layer used for the post processing camera, which
    // will be attached to the post processing camera and 2d quad.
    let layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        layer,
    ));


    commands.spawn((
        Name::new("post_processing_camera"),
        Camera2dBundle {
            camera: Camera {
                // renders after the first main camera which has default value: 0.
                order: 1,
                hdr: true,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        BloomSettings {
            intensity: 0.1,
            ..default()
        },
        layer
    ));
}
