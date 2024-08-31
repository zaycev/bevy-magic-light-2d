use std::time::Duration;

use bevy::{
    app::{App, Startup},
    asset::AssetPlugin,
    color::{palettes, Color},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseWheel,
    prelude::*,
    render::{
        camera::RenderTarget,
        texture::{ImageFilterMode, ImageSamplerDescriptor},
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
    time::common_conditions::on_timer,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_magic_light_2d::{
    gi::{render_layer::ALL_LAYERS, BevyMagicLight2DPlugin},
    prelude::{
        setup_post_processing_camera, BevyMagicLight2DSettings, CameraTargets, LightOccluder2D,
        LightPassParams, OmniLightSource2D, SkylightLight2D, SkylightMask2D, CAMERA_LAYER_FLOOR,
        CAMERA_LAYER_OBJECTS, CAMERA_LAYER_WALLS,
    },
    FloorCamera, ObjectsCamera, SpriteCamera, WallsCamera,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

/// the width and height of the map, in terms of tiles
pub const MAP_SIZE: u32 = 100;
pub const TILE_SIZE: f32 = 16.0;
pub const SPRITE_SCALE: f32 = 4.0;
pub const Z_BASE_FLOOR: f32 = 100.0; // Base z-coordinate for 2D layers.
pub const Z_BASE_OBJECTS: f32 = 200.0; // Ground object sprites.
pub const SCREEN_SIZE: (f32, f32) = (1280.0, 720.0);
pub const CAMERA_SCALE: f32 = 1.0;
pub const CAMERA_SCALE_BOUNDS: (f32, f32) = (1., 20.);
pub const CAMERA_ZOOM_SPEED: f32 = 3.;

fn main() {
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::srgba_u8(0, 0, 0, 0)))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: SCREEN_SIZE.into(),
                        title: "Bevy Magic Light 2D: Krypta Example".into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        ..default()
                    },
                }),
            BevyMagicLight2DPlugin,
            ResourceInspectorPlugin::<BevyMagicLight2DSettings>::new(),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            },
            ..default()
        })
        .register_type::<LightOccluder2D>()
        .register_type::<OmniLightSource2D>()
        .register_type::<SkylightMask2D>()
        .register_type::<SkylightLight2D>()
        .register_type::<BevyMagicLight2DSettings>()
        .register_type::<LightPassParams>()
        .add_systems(
            Startup,
            (
                (setup, setup_post_processing_camera).chain(),
                create_debug_text,
            ),
        )
        .add_systems(
            Update,
            (system_move_camera, system_camera_zoom, update_fps_text),
        )
        .run();
}

#[derive(Component)]
pub struct Candle;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct CurrentFpsText;

#[derive(Component)]
pub struct MovingFpsText;

#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_targets: Res<CameraTargets>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Utility functions to compute Z coordinate for floor and ground objects.
    let get_floor_z = |y| -> f32 { Z_BASE_FLOOR - y / SCREEN_SIZE.1 };
    let get_object_z = |y: f32| -> f32 { Z_BASE_OBJECTS - y / SCREEN_SIZE.1 };

    let block_size = Vec2::splat(TILE_SIZE * SPRITE_SCALE);
    let center_offset =
        Vec2::new(-1024.0, 1024.0) / 2.0 + block_size / 2.0 - Vec2::new(0.0, block_size.y);

    let get_block_translation = |i: u32, j: u32| {
        center_offset + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y)
    };

    // Load floor tiles.
    let floor_atlas_rows = 4;
    let floor_atlas_cols = 4;
    let floor_atlas_size = UVec2::new(16, 16);
    let floor_image = asset_server.load("art/atlas_floor.png");
    let floor_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        floor_atlas_size,
        floor_atlas_cols,
        floor_atlas_rows,
        None,
        None,
    ));

    let decorations_image = asset_server.load("art/atlas_decoration.png");
    let mut decorations_atlas = TextureAtlasLayout::new_empty(UVec2::new(256, 256));
    let candle_rect = decorations_atlas.add_texture(URect {
        min: UVec2::new(0, 0),
        max: UVec2::new(16, 16),
    });
    let decorations_atlas_handle = texture_atlases.add(decorations_atlas);

    // Load wall tiles.
    let wall_atlas_rows = 5;
    let wall_atlas_cols = 6;
    let wall_atlas_size = UVec2::new(16, 16);
    let wall_image = asset_server.load("art/atlas_wall.png");
    let wall_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        wall_atlas_size,
        wall_atlas_cols,
        wall_atlas_rows,
        None,
        None,
    ));

    let occluder_data = LightOccluder2D {
        h_size: block_size / 2.0,
    };

    // Spawn floors, walls and lights

    let mut rng = thread_rng();
    let mut floor_tiles = vec![];
    let mut walls = vec![];
    let mut decorations = vec![];

    for tx in 0..=MAP_SIZE {
        for ty in 0..MAP_SIZE {
            let xy = get_block_translation(tx, ty);
            let z = get_floor_z(xy.y);

            floor_tiles.push(
                commands
                    .spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: Vec3::new(xy.x, xy.y, z),
                                scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                                ..default()
                            },
                            texture: floor_image.clone(),
                            ..default()
                        },
                        TextureAtlas {
                            layout: floor_atlas.clone(),
                            index: rng.gen_range(0..(floor_atlas_cols * floor_atlas_rows)) as usize,
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR))
                    .id(),
            );

            match rng.gen_range(0..=5) {
                1 => {
                    let will_jitter = rng.gen_range(0..=2);
                    let potential_jitter = match will_jitter {
                        1 => OmniLightSource2D {
                            jitter_intensity: 2.5,
                            jitter_translation: 8.0,
                            ..default()
                        },
                        _ => OmniLightSource2D { ..default() },
                    };

                    decorations.push(
                        commands
                            .spawn((
                                SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(xy.x, xy.y, get_object_z(xy.y)),
                                        scale: Vec2::splat(4.0).extend(0.0),
                                        ..default()
                                    },
                                    sprite: Sprite {
                                        color: Color::srgb_u8(180, 180, 180),
                                        ..default()
                                    },
                                    texture: decorations_image.clone(),
                                    ..default()
                                },
                                Candle,
                                OmniLightSource2D {
                                    intensity: 0.5,
                                    color: Color::srgb_u8(137, 79, 24),
                                    falloff: Vec3::new(50.0, 20.0, 0.05),
                                    ..potential_jitter
                                },
                                TextureAtlas {
                                    layout: decorations_atlas_handle.clone(),
                                    index: candle_rect,
                                },
                            ))
                            .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                            .insert(LightOccluder2D {
                                h_size: Vec2::splat(2.0),
                            })
                            .insert(Name::new("candle_1"))
                            .id(),
                    );
                }
                2 => {
                    walls.push(
                        commands
                            .spawn((
                                SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(xy.x, xy.y, z),
                                        scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                                        ..default()
                                    },
                                    texture: wall_image.clone(),
                                    ..default()
                                },
                                Wall,
                                TextureAtlas {
                                    layout: wall_atlas.clone(),
                                    index: (wall_atlas_cols * 4 + 0) as usize,
                                },
                            ))
                            .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS))
                            .insert(occluder_data)
                            .id(),
                    );
                }
                _ => {} // Do nothing
            }
        }
    }

    commands
        .spawn(Name::new("floor_tiles"))
        .insert(SpatialBundle::default())
        .push_children(&floor_tiles);

    commands
        .spawn(Name::new("decorations"))
        .insert(SpatialBundle::default())
        .push_children(&walls);

    commands
        .spawn(Name::new("walls"))
        .insert(SpatialBundle::default())
        .push_children(&decorations);

    // Add skylight light.
    commands.spawn((
        SkylightLight2D {
            color: Color::srgb_u8(93, 158, 179),
            intensity: 0.025,
        },
        Name::new("global_skylight"),
    ));

    let projection = OrthographicProjection {
        scale: CAMERA_SCALE,
        near: -1000.0,
        far: 1000.0,
        ..default()
    };

    // Setup separate camera for floor, walls and objects.
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    target: RenderTarget::Image(camera_targets.floor_target.clone()),
                    ..default()
                },
                projection: projection.clone(),
                ..default()
            },
            Name::new("floors_target_camera"),
        ))
        .insert(SpriteCamera)
        .insert(FloorCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR));
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    target: RenderTarget::Image(camera_targets.walls_target.clone()),
                    ..default()
                },
                projection: projection.clone(),
                ..default()
            },
            Name::new("walls_target_camera"),
        ))
        .insert(SpriteCamera)
        .insert(WallsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS));
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    target: RenderTarget::Image(camera_targets.objects_target.clone()),
                    ..default()
                },
                projection: projection.clone(),
                ..default()
            },
            Name::new("objects_targets_camera"),
        ))
        .insert(SpriteCamera)
        .insert(ObjectsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS));
}

fn create_debug_text(mut commands: Commands, walls: Query<&Wall>, candles: Query<&Candle>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "FPS: ",
                        TextStyle {
                            font_size: 40.0,
                            ..default()
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE,
                        ..default()
                    }),
                ]),
                CurrentFpsText,
                RenderLayers::from_layers(ALL_LAYERS),
            ));

            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Moving FPS: ",
                        TextStyle {
                            font_size: 40.0,
                            ..default()
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE,
                        ..default()
                    }),
                ]),
                MovingFpsText,
                RenderLayers::from_layers(ALL_LAYERS),
            ));

            // Occluder count

            parent.spawn((
                TextBundle::from_sections([TextSection::new(
                    format!("Occluders: {}", walls.iter().len()),
                    TextStyle {
                        font_size: 20.0,
                        ..default()
                    },
                )]),
                RenderLayers::from_layers(ALL_LAYERS),
            ));

            // Candle count

            parent.spawn((
                TextBundle::from_sections([TextSection::new(
                    format!("Occluders: {}", candles.iter().len()),
                    TextStyle {
                        font_size: 20.0,
                        ..default()
                    },
                )]),
                RenderLayers::from_layers(ALL_LAYERS),
            ));
        });
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut current: Query<&mut Text, (With<CurrentFpsText>, Without<MovingFpsText>)>,
    mut moving: Query<&mut Text, With<MovingFpsText>>,
) {
    let mut current_text = current.single_mut();
    let mut moving_text = moving.single_mut();

    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) else {
        return;
    };

    if let Some(value) = fps.smoothed() {
        // Update the value of the second section
        current_text.sections[1].value = format!("{value:.2}");
    }

    if let Some(value) = fps.average() {
        moving_text.sections[1].value = format!("{value:.2}");
    }
}

#[rustfmt::skip]
fn system_move_camera(
    mut camera_current: Local<Vec2>,
    mut camera_target:  Local<Vec2>,
    mut query_cameras:  Query<&mut Transform, With<SpriteCamera>>,
        keyboard:       Res<ButtonInput<KeyCode>>,
) {
    let speed = 18.0;
    if keyboard.pressed(KeyCode::KeyW) { camera_target.y += speed; }
    if keyboard.pressed(KeyCode::KeyS) { camera_target.y -= speed; }
    if keyboard.pressed(KeyCode::KeyA) { camera_target.x -= speed; }
    if keyboard.pressed(KeyCode::KeyD) { camera_target.x += speed; }

    // Smooth camera.
    let blend_ratio = 0.2;
    let movement = *camera_target - *camera_current;
    *camera_current += movement * blend_ratio;

    // Update all sprite cameras.
    for mut camera_transform in query_cameras.iter_mut() {
        camera_transform.translation.x = camera_current.x;
        camera_transform.translation.y = camera_current.y;
    }
}

fn system_camera_zoom(
    mut cameras: Query<&mut OrthographicProjection, With<SpriteCamera>>,
    time: Res<Time>,
    mut scroll_event_reader: EventReader<MouseWheel>,
) {
    let mut projection_delta = 0.;

    for event in scroll_event_reader.read() {
        projection_delta += event.y * CAMERA_ZOOM_SPEED;
    }

    if projection_delta == 0. {
        return;
    }

    for mut camera in cameras.iter_mut() {
        camera.scale = (camera.scale - projection_delta * time.delta_seconds())
            .clamp(CAMERA_SCALE_BOUNDS.0, CAMERA_SCALE_BOUNDS.1);
    }
}
