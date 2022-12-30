use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy::{
    core_pipeline::bloom::BloomSettings, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle,
};
use bevy_2d_gi_experiment::gi::gi_post_processing::setup_post_processing_camera;
use bevy_2d_gi_experiment::gi::gi_post_processing::PostProcessingTarget;
use bevy_2d_gi_experiment::{
    gi::{
        self, gi_component::AmbientMask, gi_component::GiAmbientLight, GiTarget, LightOccluder,
        LightSource,
    },
    MainCamera, SCREEN_SIZE,
};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use rand::prelude::*;

// Base z-coordinate for 2D layers.
const Z_BASE_FLOOR: f32 = 100.0; // Floor sprites will be rendered with Z = 0.0 + y / MAX_Y.
const Z_BASE_OBJECTS: f32 = 200.0; // Object sprites will be rendered with Z = 1.0 + y / MAX_Y.

// Misc components.
#[derive(Component)]
pub struct MouseLight;
#[derive(Component)]
pub struct Movable;

fn main() {
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Tell the asset server to watch for asset changes on disk:
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: SCREEN_SIZE.0 as f32,
                        height: SCREEN_SIZE.1 as f32,
                        title: "Bevy Magic Light 2D: Krypta Example".into(),
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
        .add_system(system_move_light_to_cursor.after(system_move_camera))
        .add_system(system_move_target.after(system_move_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    post_processing_target: Res<PostProcessingTarget>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {

    
    commands.spawn(SpriteBundle {
        texture: asset_server.load("art/white_bg.png"),
        ..default()
    })
    .insert(Name::new("background"));

    let mut occluders = vec![];
    let occluder_entity = commands
    .spawn((
    Transform::from_translation(Vec3::new(0., 0., 0.)),
    GlobalTransform::default(),
    Visibility::VISIBLE,
    ComputedVisibility::default(),
        
        LightOccluder {
        h_size: Vec2::new(40.0, 20.0),
    }))
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
                let z = Z_BASE_OBJECTS - y / (SCREEN_SIZE.1 as f32);
                return cmd
                    .spawn(Name::new(name))
                    .insert(light_source)
                    .insert(SpatialBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, z),
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
                jitter_intensity: 1.0,
                jitter_translation: 2.0,
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
                jitter_intensity: 1.0,
                jitter_translation: 2.0,
                ..base
            },
        ));
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("lights"))
        .push_children(&lights);

    // Add ambient light.
    // commands.spawn((
    //     GiAmbientLight {
    //         color: Color::rgb_u8(93, 158, 179),
    //         intensity: 0.04,
    //     },
    //     Name::new("ambient_light"),
    // ));

    // Add light source.
    // commands
    //     .spawn(MaterialMesh2dBundle {
    //         mesh: block_mesh.clone().into(),
    //         material: materials.add(ColorMaterial::from(Color::YELLOW)).into(),
    //         transform: Transform {
    //             scale: Vec3::splat(8.0),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .insert(Name::new("cursor_light"))
    //     .insert(LightSource {
    //         intensity: 10.0,
    //         radius: 32.0,
    //         color: Color::rgb_u8(219, 104, 72),
    //         falloff: Vec3::new(50.0, 20.0, 0.05),
    //         ..default()
    //     })
    //     .insert(MouseLight);

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

fn system_move_light_to_cursor(
    mut commands: Commands,
    windows: ResMut<Windows>,
    mut query_light: Query<
        (&mut Transform, &mut LightSource),
        (Without<MainCamera>, With<MouseLight>),
    >,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut rng = rand::thread_rng();

    if let Ok((camera, camera_global_transform)) = query_camera.get_single() {
        let window_opt = if let RenderTarget::Window(id) = camera.target {
            windows.get(id)
        } else {
            windows.get_primary()
        };

        if let Some(window) = window_opt {
            if let Some(screen_pos) = window.cursor_position() {
                let window_size = Vec2::new(window.width() as f32, window.height() as f32);

                let mouse_ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
                let ndc_to_world =
                    camera_global_transform.compute_matrix() * camera.projection_matrix().inverse();
                let mouse_world = ndc_to_world.project_point3(mouse_ndc.extend(-1.0));

                if let Ok((mut mouse_transform, mut mouse_color)) = query_light.get_single_mut() {
                    mouse_transform.translation = mouse_world.truncate().extend(100.0);

                    if mouse.just_pressed(MouseButton::Right) {
                        mouse_color.color = Color::rgba(rng.gen(), rng.gen(), rng.gen(), 1.0);
    
                        log::info!("Added new light source: {:?}", mouse_color.color);
                    }
                    if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::LShift) {
                        commands
                            .spawn(SpatialBundle {
                                transform: Transform {
                                    translation: mouse_world.truncate().extend(0.0),
                                    scale: Vec3::splat(8.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Name::new("point_light"))
                            .insert(LightSource {
                                intensity: mouse_color.intensity,
                                radius: mouse_color.radius,
                                color: mouse_color.color,
                                falloff: mouse_color.falloff,
                                jitter_intensity: 0.0,
                                jitter_translation: 0.0,
                            });
                    }
                }
            }
        }
    }
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

fn system_move_target(
    mut query_targets: Query<&mut Transform, With<GiTarget>>,
    query_camera: Query<&Transform, (With<MainCamera>, Without<GiTarget>)>,
) {
    if let Ok(camera_transform) = query_camera.get_single() {
        for mut target_transform in query_targets.iter_mut() {
            target_transform.translation.x = camera_transform.translation.x;
            target_transform.translation.y = camera_transform.translation.y;
        }
    }
}
