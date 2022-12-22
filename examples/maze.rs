use bevy::{prelude::*, render::camera::RenderTarget, sprite::MaterialMesh2dBundle, core_pipeline::bloom::BloomSettings};
use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy_2d_gi_experiment::{
    gi::{self, LightOccluder, LightSource, GiTarget, gi_component::AmbientMask, gi_component::GiAmbientLight},
    MainCamera, SCREEN_SIZE,
};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use rand::prelude::*;
use bevy_2d_gi_experiment::gi::gi_post_processing::PostProcessingTarget;
use bevy_2d_gi_experiment::gi::gi_post_processing::setup_post_processing_camera;

// Maze map. 1 represents wall.
const MAZE: &[&[u8]] = &[
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0 , 0, 0, 0],
    &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0 , 0, 0, 0],
    &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
];

// Base z-coordinate for 2D layers.
const Z_BASE_FLOOR:   f32 = 100.0; // Floor sprites will be rendered with Z = 0.0 + y / MAX_Y.
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
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }).set(WindowPlugin{
            window: WindowDescriptor {
                width: SCREEN_SIZE.0 as f32,
                height: SCREEN_SIZE.1 as f32,
                title: "Bevy Magic Light 2D: Krypta Example".into(),
                resizable: false,
                mode: WindowMode::Windowed,
                ..Default::default()
             },
             ..Default::default()
        }).set(ImagePlugin{
            default_sampler: SamplerDescriptor {
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            },
        }))
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
    mut commands:               Commands,
    mut meshes:                 ResMut<Assets<Mesh>>,
    mut materials:              ResMut<Assets<ColorMaterial>>,
        post_processing_target: Res<PostProcessingTarget>,
        asset_server:           Res<AssetServer>,
    mut texture_atlases:        ResMut<Assets<TextureAtlas>>,
) {

    // Generate square occluders from MAZE array.
    let block_size = Vec2::splat(16.0 * 4.0);

    let center_offset = Vec2::new(-(SCREEN_SIZE.0 as f32) / 2.0, (SCREEN_SIZE.1 as f32) / 2.0)
        + block_size / 2.0
        - Vec2::new(0.0, block_size.y);

    let get_block_translation = |i: usize, j: usize| {
        center_offset + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y)
    };

    let block_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let mut occluders = vec![];

    // Load floor tiles.
    let floor_atlas_rows = 4;
    let floor_atlas_cols = 4;
    let floor_atlas_size = Vec2::new(16.0, 16.0);
    let floor_image = asset_server.load("art/atlas_floor.png");
    let floor_atlas = texture_atlases.add(TextureAtlas::from_grid(
        floor_image,
        floor_atlas_size,
        floor_atlas_cols,
        floor_atlas_rows,
        None,
        None,
    ));

    // Load wall tiles.
    let wall_atlas_rows = 5;
    let wall_atlas_cols = 6;
    let wall_atlas_size = Vec2::new(16.0, 16.0);
    let wall_image = asset_server.load("art/atlas_wall.png");
    let wall_atlas = texture_atlases.add(TextureAtlas::from_grid(
        wall_image,
        wall_atlas_size,
        wall_atlas_cols,
        wall_atlas_rows,
        None,
        None,
    ));

    // Load decorations.
    let decorations_image = asset_server.load("art/atlas_decoration.png");

    // Spawn floor tiles.
    let mut rng = thread_rng();
    let mut floor_tiles = vec![];
    for (i, row) in MAZE.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {

            let translation = get_block_translation(i, j);
            let z = Z_BASE_FLOOR - translation.y / (SCREEN_SIZE.1 as f32);
            let sprite_index = rng.gen_range(0..(floor_atlas_cols * floor_atlas_rows));

            floor_tiles.push(commands.spawn(SpriteSheetBundle{
                transform: Transform{
                    translation: Vec3::new(translation.x, translation.y, z),
                    scale: Vec2::splat(4.0).extend(0.0),
                    ..Default::default()
                },
                sprite: TextureAtlasSprite::new(sprite_index),
                texture_atlas: floor_atlas.clone(),
                ..Default::default()
            }).id());
        }
    }

    commands
        .spawn(Name::new("floor_tiles"))
        .insert(SpatialBundle::default())
        .push_children(&floor_tiles);

    let maze_rows = MAZE.len() as i32;
    let maze_cols = MAZE[0].len() as i32;

    ///
    ///
    let get_wall_safe = |r: i32, c: i32, offset: (i32, i32)| {
        let r1 = r + offset.0;
        let c1 = c + offset.1;
        if r1 < 0 || r1 >= maze_rows {
            return 1;
        }
        if c1 < 0 || c1 >= maze_cols {
            return 1;
        }
        return MAZE[r1 as usize][c1 as usize];
    };

    ///
    ///
    let get_wall_sprite_index = |r: usize, c: usize| {
        let r = r as i32;
        let c = c as i32;

        let w_up    = get_wall_safe(r, c, (-1,  0));
        let w_down  = get_wall_safe(r, c, ( 1,  0));
        let w_left  = get_wall_safe(r, c, ( 0, -1));
        let w_right = get_wall_safe(r, c, ( 0,  1));

        let total_walls = w_up   +
                          w_down +
                          w_left +
                          w_right;

        if total_walls == 4 {
            return wall_atlas_cols * 0 + 0;
        }

        if total_walls == 3 {
            if w_up == 0 {
                return wall_atlas_cols * 1 + 0;
            }
            if w_left == 0 {
                return wall_atlas_cols * 1 + 1;
            }
            if w_down == 0 {
                return wall_atlas_cols * 1 + 2;
            }
            if w_right == 0 {
                return wall_atlas_cols * 1 + 3;
            }
        }

        if total_walls == 2 {
            if w_left == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 0;
            }

            if w_up == 1 && w_down == 1 {
                return wall_atlas_cols * 2 + 1;
            }

            if w_up == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 2;
            }

            if w_down == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 3;
            }

            if w_up == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 4;
            }

            if w_down == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 5;
            }
        }

        if total_walls == 1 {
            if w_left == 1 {
                return wall_atlas_cols * 3 + 0;
            }
            if w_down == 1 {
                return wall_atlas_cols * 3 + 1;
            }
            if w_up == 1 {
                return wall_atlas_cols * 3 + 2;
            }
            if w_right == 1 {
                return wall_atlas_cols * 3 + 3;
            }
        }

        return wall_atlas_cols * 4 + 0;
    };

    // Add light occluders for maze stored.
    for (i, row) in MAZE.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if *cell == 1 {

                let translation = get_block_translation(i, j);
                let z = Z_BASE_OBJECTS - translation.y / (SCREEN_SIZE.1 as f32);

                let mut occluder_sprite = TextureAtlasSprite::new(get_wall_sprite_index(i, j));
                occluder_sprite.color = Color::rgb(0.5, 0.5, 0.5);
                let occluder_entity = commands
                    .spawn(SpriteSheetBundle{
                        transform: Transform{
                            translation: Vec3::new(translation.x, translation.y, z),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..Default::default()
                        },
                        sprite: occluder_sprite,
                        texture_atlas: wall_atlas.clone(),
                        ..Default::default()
                    })
                    .insert(LightOccluder {
                        h_size: block_size / 2.0,
                    })
                    .id();

                occluders.push(occluder_entity);
            }
        }
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("occluders"))
        .push_children(&occluders);

    // Add decorations.
    // let mut decorations = vec![];
    {
        let get_z = |y: f32| Z_BASE_OBJECTS - y / (SCREEN_SIZE.1 as f32);
        let mut decorations_atlas = TextureAtlas::new_empty(decorations_image, Vec2::new(256.0, 256.0));

        let candle_1 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(16.0, 16.0),
        });
        let candle_2 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(16.0, 0.0),
            max: Vec2::new(32.0, 16.0),
        });
        let candle_3 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(32.0, 0.0),
            max: Vec2::new(48.0, 16.0),
        });
        let candle_4 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(48.0, 0.0),
            max: Vec2::new(64.0, 16.0),
        });
        let thomb_1 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(32.0, 16.0),
            max: Vec2::new(80.0, 48.0),
        });
        let sewerage_1 = decorations_atlas.add_texture(Rect{
            min: Vec2::new(0.0, 16.0),
            max: Vec2::new(32.0, 48.0),
        });


        let texture_atlas_handle = texture_atlases.add(decorations_atlas);


        // Candle 1.
        {
            let x = 100.0;
            let y = -388.5;
            let mut sprite       = TextureAtlasSprite::new(candle_1);
            sprite.color = Color::rgb_u8(120, 120, 120);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                }) .insert(LightOccluder {
                h_size: Vec2::splat(2.0),
            });
        }

        // Candle 2.
        {
            let x = -32.1;
            let y = -384.2;
            let mut sprite       = TextureAtlasSprite::new(candle_2);
            sprite.color = Color::rgb_u8(120, 120, 120);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                }) .insert(LightOccluder {
                h_size: Vec2::splat(2.0),
            });
        }


        // Candle 3.
        {
            let x = -351.5;
            let y = -126.0;
            let mut sprite       = TextureAtlasSprite::new(candle_3);
            sprite.color = Color::rgb_u8(120, 120, 120);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                }) .insert(LightOccluder {
                h_size: Vec2::splat(2.0),
            });
        }


        // Candle 3.
        {
            let x = 413.0;
            let y = -124.6;
            let mut sprite       = TextureAtlasSprite::new(candle_4);
            sprite.color = Color::rgb_u8(120, 120, 120);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                }) .insert(LightOccluder {
                h_size: Vec2::splat(2.0),
            });
        }

        // Tomb 1.
        {
            let x = 31.5;
            let y = -220.0;
            let mut sprite       = TextureAtlasSprite::new(thomb_1);
            sprite.color = Color::rgb_u8(255, 255, 255);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                }).insert(LightOccluder {
                    h_size: Vec2::new(72.8, 31.0),
                });
        }

        // Sewerage 1.
        {
            let x = 31.5;
            let y = -38.5;
            let mut sprite       = TextureAtlasSprite::new(sewerage_1);
            sprite.color = Color::rgb_u8(255, 255, 255);
            commands
                .spawn(SpriteSheetBundle{
                    transform: Transform{
                        translation: Vec3::new(x, y, get_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..Default::default()
                    },
                    sprite:        sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..Default::default()
                });
        }



    }

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light = |cmd: &mut Commands, x: f32, y: f32, name: &'static str, light_source: LightSource | {
            let z = Z_BASE_OBJECTS -y / (SCREEN_SIZE.1 as f32);
            return cmd.spawn(Name::new(name)).insert(light_source).insert(SpatialBundle{
                transform: Transform {
                    translation: Vec3::new(x, y, z),
                    ..default()
                },
                ..default()
            }).id();
        };

        let base = LightSource {falloff: Vec3::new(50.0, 20.0, 0.05), intensity: 10.0, ..default()};
        lights.push(spawn_light(&mut commands, 90.667,  -387.333, "outdoor_krypta_torch_1", LightSource {intensity: 6.0,  color: Color::rgb_u8(137, 79,  24),  jitter_intensity: 1.0,  jitter_translation: 2.0, ..base}));
        lights.push(spawn_light(&mut commands,-36.000,  -387.333, "outdoor_krypta_torch_2", LightSource {intensity: 6.0,  color: Color::rgb_u8(137, 79,  24),  jitter_intensity: 1.0,  jitter_translation: 2.0, ..base}));
        lights.push(spawn_light(&mut commands, 247.333, -302.667, "indoor_krypta_light_1",  LightSource {intensity: 10.0, color: Color::rgb_u8(76,  57,  211), jitter_intensity: 2.0,  jitter_translation: 0.0, ..base}));
        lights.push(spawn_light(&mut commands,-172.000, -302.333, "indoor_krypta_light_2",  LightSource {intensity: 10.0, color: Color::rgb_u8(76,  57,  211), jitter_intensity: 2.0,  jitter_translation: 0.0, ..base}));
        lights.push(spawn_light(&mut commands,-352.000, -122.000, "outdoor_krypta_torch_3", LightSource {intensity: 6.0,  color: Color::rgb_u8(137, 79,  24),  jitter_intensity: 1.0,  jitter_translation: 2.0, ..base}));
        lights.push(spawn_light(&mut commands, 410.667, -118.667, "outdoor_krypta_torch_4", LightSource {intensity: 6.0,  color: Color::rgb_u8(137, 79,  24),  jitter_intensity: 1.0,  jitter_translation: 2.0, ..base}));
        lights.push(spawn_light(&mut commands, 28.0,    -34.0,    "indoor_krypta_ghost_1",  LightSource {intensity: 0.8,  color: Color::rgb_u8(6,   53,  6),   jitter_intensity: 0.2,  jitter_translation: 0.0, ..base}));
        lights.push(spawn_light(&mut commands, 31.392,  -168.3,   "indoor_krypta_tomb_1",   LightSource {intensity: 0.4,  color: Color::rgb_u8(252, 182, 182), jitter_intensity: 0.05, jitter_translation: 4.7, ..base}));

    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("lights"))
        .push_children(&lights);


    // Add roof.
    commands
        .spawn(SpatialBundle{
            transform: Transform {
                translation: Vec3::new(30.0, -180.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("ambient_mask"))
        .insert(AmbientMask {
            h_size: Vec2::new(430.0, 330.0)
        });

    // Add ambient light.
    commands.spawn((GiAmbientLight {
        color: Color::rgb_u8(93, 158, 179),
        intensity: 0.04,
    }, Name::new("ambient_light")));

    // Add light source.
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: block_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)).into(),
            transform: Transform {
                scale: Vec3::splat(8.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("cursor_light"))
        .insert(LightSource {
            intensity: 10.0,
            radius: 32.0,
            color: Color::rgb_u8(219, 104, 72),
            falloff: Vec3::new(50.0, 20.0, 0.05),
            ..default()
        })
        .insert(MouseLight);

    let render_target = post_processing_target.handle.clone().expect("No post processing target");

    commands
        .spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                priority: 0,
                target: RenderTarget::Image(render_target),
                ..Default::default()
            },
            ..Default::default()
        }, Name::new("main_camera")))
        .insert(MainCamera)
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
}

fn system_move_light_to_cursor(
    mut commands:       Commands,
        windows:        ResMut<Windows>,
    mut query_light:    Query<(
            &mut Transform,
            &mut LightSource), (
                Without<MainCamera>,
                With<MouseLight>
            )
        >,
        query_camera:   Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mouse:          Res<Input<MouseButton>>,
        keyboard:       Res<Input<KeyCode>>,
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

                let (mut mouse_transform, mut mouse_color) =  query_light.single_mut();

                mouse_transform.translation = mouse_world.truncate().extend(100.0);

                if mouse.just_pressed(MouseButton::Right) {

                    mouse_color.color = Color::rgba(
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        1.0,
                    );

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
                            radius:    mouse_color.radius,
                            color:     mouse_color.color,
                            falloff:   mouse_color.falloff,
                            jitter_intensity: 0.0,
                            jitter_translation: 0.0,
                        });
                }
            }
        }
    }
}


fn system_move_camera(
    mut query_camera:   Query<&mut Transform, With<MainCamera>>,
        keyboard:       Res<Input<KeyCode>>,
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
    mut query_targets:  Query<&mut Transform, With<GiTarget>>,
        query_camera:  Query<&Transform, (With<MainCamera>, Without<GiTarget>)>,
) {

    if let Ok(camera_transform) = query_camera.get_single() {
        for mut target_transform in query_targets.iter_mut() {
            target_transform.translation.x = camera_transform.translation.x;
            target_transform.translation.y = camera_transform.translation.y;
        }
    }
}