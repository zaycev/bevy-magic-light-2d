use bevy::core_pipeline::bloom::BloomSettings;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use rand::prelude::*;

use bevy_magic_light_2d::gi::gi_component::{AmbientMask, GiAmbientLight};
use bevy_magic_light_2d::gi::gi_post_processing::{
    setup_post_processing_camera, PostProcessingTarget,
};
use bevy_magic_light_2d::gi::{self, GiTarget, LightOccluder, LightSource};
use bevy_magic_light_2d::{MainCamera, SCREEN_SIZE};

fn main() {
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: SCREEN_SIZE.0 as f32,
                        height: SCREEN_SIZE.1 as f32,
                        title: "Bevy Magic Light 2D: Minimal Example".into(),
                        resizable: false,
                        mode: WindowMode::Windowed,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        mag_filter: FilterMode::Nearest,
                        min_filter: FilterMode::Nearest,
                        ..Default::default()
                    },
                }),
        )
        .add_plugin(gi::GiComputePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<LightOccluder>()
        .register_inspectable::<LightSource>()
        .register_inspectable::<AmbientMask>()
        .register_inspectable::<GiAmbientLight>()
        .register_type::<BloomSettings>()
        .add_startup_system(setup.after(setup_post_processing_camera))
        .add_system(system_move_camera)
        .run();
}

fn setup(mut commands: Commands, post_processing_target: Res<PostProcessingTarget>) {
    let mut occluders = vec![];
    let occluder_entity = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            GlobalTransform::default(),
            Visibility::VISIBLE,
            ComputedVisibility::default(),
            LightOccluder {
                h_size: Vec2::new(40.0, 20.0),
            },
        ))
        .id();

    occluders.push(occluder_entity);

    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("occluders"))
        .push_children(&occluders);

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light =
            |cmd: &mut Commands, x: f32, y: f32, name: &'static str, light_source: LightSource| {
                return cmd
                    .spawn(Name::new(name))
                    .insert(light_source)
                    .insert(SpatialBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, 0.0),
                            ..default()
                        },
                        ..default()
                    })
                    .id();
            };

        let base = LightSource {
            falloff: Vec3::new(10., 10., 0.05),
            intensity: 10.0,
            ..default()
        };

        lights.push(spawn_light(
            &mut commands,
            -512.,
            -512.,
            "left",
            LightSource {
                intensity: 6.0,
                color: Color::rgb_u8(255, 255, 0),
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            512.,
            -512.,
            "right",
            LightSource {
                intensity: 6.0,
                color: Color::rgb_u8(0, 255, 255),
                ..base
            },
        ));
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("lights"))
        .push_children(&lights);

    let render_target = post_processing_target
        .handle
        .clone()
        .expect("No post processing target");

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    priority: 0,
                    target: RenderTarget::Image(render_target),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("main_camera"),
        ))
        .insert(MainCamera)
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
}

fn system_move_camera(
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if let Ok(mut camera_transform) = query_camera.get_single_mut() {
        let speed = 10.0;

        if keyboard.pressed(KeyCode::W) {
            camera_transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::S) {
            camera_transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::A) {
            camera_transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::D) {
            camera_transform.translation.x += speed;
        }
    }
}
