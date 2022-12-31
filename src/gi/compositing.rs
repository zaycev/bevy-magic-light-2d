use bevy::core_pipeline::bloom::BloomSettings;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, MaterialMesh2dBundle};

use crate::gi::pipeline::PipelineTargetsWrapper;
use crate::gi::resource::ComputedTargetSizes;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "bc2f08eb-a0fb-43f1-a908-54871ea597d5"]
pub struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    irradiance_image: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct PostProcessingTarget {
    pub handle: Option<Handle<Image>>,
}

impl Material2d for PostProcessingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gi_post_processing.wgsl".into()
    }
}

pub fn setup_post_processing_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut post_processing_target: ResMut<PostProcessingTarget>,

    gpu_targets_sizes: Res<ComputedTargetSizes>,
    gpu_targets_wrapper: Res<PipelineTargetsWrapper>,
) {
    let window_size = Extent3d {
        width: gpu_targets_sizes.primary_target_usize.x,
        height: gpu_targets_sizes.primary_target_usize.y,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("Post Processing Image"),
            size: window_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // Fill image data with zeroes.
    image.resize(window_size);

    // This specifies the layer used for the post processing camera, which
    // will be attached to the post processing camera and 2d quad.
    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);
    let image_handle = images.add(image);

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        gpu_targets_sizes.primary_target_size.x,
        gpu_targets_sizes.primary_target_size.y,
    ))));

    // This material has the texture that has been rendered.
    post_processing_target.handle = Some(image_handle.clone());
    let material_handle = post_processing_materials.add(PostProcessingMaterial {
        source_image: image_handle,
        irradiance_image: gpu_targets_wrapper
            .targets
            .as_ref()
            .expect("Targets must be initialized")
            .ss_filter_target
            .clone(),
    });

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
        post_processing_pass_layer,
    ));

    commands.spawn((
        Name::new("post_processing_camera"),
        Camera2dBundle {
            camera: Camera {
                // renders after the first main camera which has default value: 0.
                priority: 1,
                hdr: true,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        BloomSettings {
            intensity: 0.1,
            ..default()
        },
        post_processing_pass_layer,
    ));
}
