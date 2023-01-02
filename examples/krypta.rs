use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy::render::view::RenderLayers;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::prelude::*;
use bevy_magic_light_2d::prelude::*;
use rand::prelude::*;

pub const TILE_SIZE: f32 = 16.0;
pub const SPRITE_SCALE: f32 = 4.0;
pub const Z_BASE_FLOOR: f32 = 100.0; // Base z-coordinate for 2D layers.
pub const Z_BASE_OBJECTS: f32 = 200.0; // Ground object sprites.
pub const SCREEN_SIZE: (f32, f32) = (1080.0 * 1.2, 1920.0 * 1.2);

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
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: SCREEN_SIZE.0,
                        height: SCREEN_SIZE.1,
                        title: "Bevy Magic Light 2D: Krypta Example".into(),
                        resizable: false,
                        mode: WindowMode::Windowed,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        mag_filter: FilterMode::Nearest,
                        min_filter: FilterMode::Nearest,
                        ..default()
                    },
                }),
        )
        .add_plugin(BevyMagicLight2DPlugin)
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            },
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InspectorPlugin::<BevyMagicLight2DSettings>::new())
        .register_inspectable::<LightOccluder2D>()
        .register_inspectable::<OmniLightSource2D>()
        .register_inspectable::<SkylightMask2D>()
        .register_inspectable::<SkylightLight2D>()
        .register_inspectable::<BevyMagicLight2DSettings>()
        .add_startup_system(setup.after(setup_post_processing_camera))
        .add_system(system_move_camera)
        .add_system(system_control_mouse_light.after(system_move_camera))
        .run();
}

#[rustfmt::skip]
fn setup(
    mut commands:               Commands,
    mut meshes:                 ResMut<Assets<Mesh>>,
    mut materials:              ResMut<Assets<ColorMaterial>>,
        post_processing_target: Res<PostProcessingTarget>,
        asset_server:           Res<AssetServer>,
    mut texture_atlases:        ResMut<Assets<TextureAtlas>>,
) {

    // Utility functions to compute Z coordinate for floor and ground objects.
    let get_floor_z  = | y | -> f32 { Z_BASE_FLOOR   - y / (SCREEN_SIZE.1 as f32) };
    let get_object_z = | y | -> f32 { Z_BASE_OBJECTS - y / (SCREEN_SIZE.1 as f32) };

    // Maze map. 1 represents wall.
    let walls_info: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    // Generate square occluders from walls_info.
    let block_size    = Vec2::splat(TILE_SIZE * SPRITE_SCALE);
    let center_offset = Vec2::new(-1024.0, 1024.0) / 2.0
                      + block_size / 2.0
                      - Vec2::new(0.0, block_size.y);

    let get_block_translation = |i: usize, j: usize| {
        center_offset + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y)
    };

    let block_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let mut walls = vec![];

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

    // Load decoration sprites.
    let decorations_image = asset_server.load("art/atlas_decoration.png");

    // Spawn floor tiles.
    let mut rng = thread_rng();
    let mut floor_tiles = vec![];
    for (i, row) in walls_info.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {

            let xy = get_block_translation(i, j);
            let z  = get_floor_z(xy.y);
            let id = rng.gen_range(0..(floor_atlas_cols * floor_atlas_rows));

            floor_tiles.push( commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(xy.x, xy.y, z),
                        scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                        ..default()
                    },
                    sprite: TextureAtlasSprite::new(id),
                    texture_atlas: floor_atlas.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR)).id());
        }
    }

    commands
        .spawn(Name::new("floor_tiles"))
        .insert(SpatialBundle::default())
        .push_children(&floor_tiles);

    let maze_rows = walls_info.len() as i32;
    let maze_cols = walls_info[0].len() as i32;

    // Get wall value clamping to edge.
    let get_wall_safe = |r: i32, c: i32, offset: (i32, i32)| {
        let r1 = r + offset.0;
        let c1 = c + offset.1;
        if r1 < 0 || r1 >= maze_rows {
            return 1;
        }
        if c1 < 0 || c1 >= maze_cols {
            return 1;
        }
        return walls_info[r1 as usize][c1 as usize];
    };

    // Get atlas sprite index for wall.
    let get_wall_sprite_index = |r: usize, c: usize| {
        let r = r as i32;
        let c = c as i32;

        let w_up    = get_wall_safe(r, c, (-1,  0));
        let w_down  = get_wall_safe(r, c, ( 1,  0));
        let w_left  = get_wall_safe(r, c, ( 0, -1));
        let w_right = get_wall_safe(r, c, ( 0,  1));

        let total_walls = w_up + w_down + w_left + w_right;

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

    // Add walls with occluder component.
    let occluder_data = LightOccluder2D { h_size: block_size / 2.0 };
    for (i, row) in walls_info.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if *cell == 1 {
                let xy = get_block_translation(i, j);
                let z  = get_object_z(xy.y);
                let id = get_wall_sprite_index(i, j);

                walls.push(commands.spawn(SpriteSheetBundle {
                        transform: Transform {
                            translation: Vec3::new(xy.x, xy.y, z),
                            scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                            ..default()
                        },
                        sprite: TextureAtlasSprite::new(id),
                        texture_atlas: wall_atlas.clone(),
                        ..default()
                    })
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS))
                    .insert(occluder_data.clone()).id());
            }
        }
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("walls"))
        .push_children(&walls);

    // Add decorations.
    // TODO: consider adding some utility function to avoid code duplication.
    let mut decorations = vec![];
    {
        let mut decorations_atlas = TextureAtlas::new_empty(
            decorations_image,
            Vec2::new(256.0, 256.0));

        let candle_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(16.0, 16.0),
        });
        let candle_rect_2 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(16.0, 0.0),
            max: Vec2::new(32.0, 16.0),
        });
        let candle_rect_3 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(32.0, 0.0),
            max: Vec2::new(48.0, 16.0),
        });
        let candle_rect_4 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(48.0, 0.0),
            max: Vec2::new(64.0, 16.0),
        });
        let tomb_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(32.0, 16.0),
            max: Vec2::new(80.0, 48.0),
        });
        let sewerage_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(0.0, 16.0),
            max: Vec2::new(32.0, 48.0),
        });

        let texture_atlas_handle = texture_atlases.add(decorations_atlas);

        // Candle 1.
        {
            let x = 100.0;
            let y = -388.5;
            let mut sprite = TextureAtlasSprite::new(candle_rect_1);
            sprite.color = Color::rgb_u8(120, 120, 120);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_1")).id());

        }

        // Candle 2.
        {
            let x = -32.1;
            let y = -384.2;
            let mut sprite = TextureAtlasSprite::new(candle_rect_2);
            sprite.color = Color::rgb_u8(120, 120, 120);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_2")).id());
        }

        // Candle 3.
        {
            let x = -351.5;
            let y = -126.0;
            let mut sprite = TextureAtlasSprite::new(candle_rect_3);
            sprite.color = Color::rgb_u8(120, 120, 120);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_3")).id());
        }

        // Candle 4.
        {
            let x = 413.0;
            let y = -124.6;
            let mut sprite = TextureAtlasSprite::new(candle_rect_4);
            sprite.color = Color::rgb_u8(120, 120, 120);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_4")).id());
        }

        // Tomb 1.
        {
            let x = 31.5;
            let y = -220.0;
            let mut sprite = TextureAtlasSprite::new(tomb_rect_1);
            sprite.color = Color::rgb_u8(255, 255, 255);
            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::new(72.8, 31.0),
                })
                .insert(Name::new("tomb_1")).id());
        }

        // Sewerage 1.
        {
            let x = 31.5;
            let y = -38.5;
            let mut sprite = TextureAtlasSprite::new(sewerage_rect_1);
            sprite.color = Color::rgb_u8(255, 255, 255);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(Name::new("sewerage_1")).id());
        }
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("decorations"))
        .push_children(&decorations);

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light =
            |cmd: &mut Commands, x: f32, y: f32, name: &'static str, light_source: OmniLightSource2D| {
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
                    .insert(RenderLayers::all())
                    .id();
            };

        let base = OmniLightSource2D {
            falloff: Vec3::new(50.0, 20.0, 0.05),
            intensity: 10.0,
            ..default()
        };
        lights.push(spawn_light(
            &mut commands,
            90.667,
            -393.8,
            "outdoor_krypta_torch_1",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -36.000,
            -393.8,
            "outdoor_krypta_torch_2",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            230.9,
            -284.6,
            "indoor_krypta_light_1",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -163.5,
            -292.7,
            "indoor_krypta_light_2",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -352.000,
            -131.2,
            "outdoor_krypta_torch_3",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            410.667,
            -141.8,
            "outdoor_krypta_torch_4",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            28.0,
            -34.0,
            "indoor_krypta_ghost_1",
            OmniLightSource2D {
                intensity: 0.8,
                color: Color::rgb_u8(6, 53, 6),
                jitter_intensity: 0.2,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            31.392,
            -168.3,
            "indoor_krypta_tomb_1",
            OmniLightSource2D {
                intensity: 0.4,
                color: Color::rgb_u8(252, 182, 182),
                jitter_intensity: 0.05,
                jitter_translation: 4.7,
                ..base
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            10.385,
            -1170.82,
            "outdoor_light_9",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(0, 206, 94),
                jitter_intensity: 0.0,
                jitter_translation: 8.0,
                ..base
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            182.375,
            -1170.82,
            "outdoor_light_10",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(0, 206, 94),
                jitter_intensity: 0.0,
                jitter_translation: 8.0,
                ..base
            },
        ));

    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("lights"))
        .push_children(&lights);

    // Add roofs.
    commands
        .spawn(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(30.0, -180.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("skylight_mask_1"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(430.0, 330.0),
        });
    commands
        .spawn(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(101.6, -989.4, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("skylight_mask_2"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(163.3, 156.1),
        });

    // Add skylight light.
    commands.spawn((
        SkylightLight2D {
            color: Color::rgb_u8(93, 158, 179),
            intensity: 0.025,
        },
        Name::new("global_skylight"),
    ));

    // Add light source.
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: block_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)).into(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1000.0),
                scale:       Vec3::splat(8.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("cursor_light"))
        .insert(OmniLightSource2D {
            intensity: 10.0,
            color:     Color::rgb_u8(254, 100, 34),
            falloff:   Vec3::new(50.0, 20.0, 0.05),
            ..default()
        })
        .insert(RenderLayers::all())
        .insert(MouseLight);

    let (floor_target, walls_target, objects_target) = post_processing_target
        .handles
        .clone()
        .expect("No post processing target");


    // Setup separate camera for floor, walls and objects.
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    priority: 0,
                    target: RenderTarget::Image(floor_target),
                    ..default()
                },
                ..default()
            },
            Name::new("main_camera_floor"),
        ))
        .insert(SpriteCamera)
        .insert(FloorCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    priority: 0,
                    target: RenderTarget::Image(walls_target),
                    ..default()
                },
                ..default()
            },
            Name::new("main_camera_walls"),
        ))
        .insert(SpriteCamera)
        .insert(WallsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    priority: 0,
                    target: RenderTarget::Image(objects_target),
                    ..default()
                },
                ..default()
            },
            Name::new("main_camera_objects"),
        ))
        .insert(SpriteCamera)
        .insert(ObjectsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });


}

#[rustfmt::skip]
fn system_control_mouse_light(
    mut commands:      Commands,
        windows:       ResMut<Windows>,
    mut query_light:   Query<(&mut Transform, &mut OmniLightSource2D), With<MouseLight>>,
        query_cameras: Query<(&Camera, &GlobalTransform), With<SpriteCamera>>,
        mouse:         Res<Input<MouseButton>>,
        keyboard:      Res<Input<KeyCode>>,
) {
    let mut rng = thread_rng();

    // We only need to iter over first camera matched.
    for (camera, camera_transform) in query_cameras.iter() {

        let window_opt = if let RenderTarget::Window(id) = camera.target {
            windows.get(id)
        } else {
            windows.get_primary()
        };

        if let Some(window) = window_opt {
            if let Some(screen_pos) = window.cursor_position() {
                let window_size  = Vec2::new(window.width() as f32, window.height() as f32);
                let mouse_ndc    = (screen_pos / window_size) * 2.0 - Vec2::ONE;
                let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
                let mouse_world  = ndc_to_world.project_point3(mouse_ndc.extend(-1.0));

                let (mut mouse_transform, mut mouse_color) = query_light.single_mut();
                mouse_transform.translation = mouse_world.truncate().extend(1000.0);

                if mouse.just_pressed(MouseButton::Right) {
                    mouse_color.color = Color::rgba(rng.gen(), rng.gen(), rng.gen(), 1.0);
                }
                if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::LShift) {
                    commands
                        .spawn(SpatialBundle {
                            transform: Transform {
                                translation: mouse_world.truncate().extend(0.0),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(Name::new("point_light"))
                        .insert(OmniLightSource2D {
                            jitter_intensity: 0.0,
                            jitter_translation: 0.0,
                            ..*mouse_color
                        });
                }
            }
        }

        break;
    }
}

#[rustfmt::skip]
fn system_move_camera(
    mut camera_current: Local<Vec2>,
    mut camera_target:  Local<Vec2>,
    mut query_cameras:  Query<&mut Transform, With<SpriteCamera>>,
        keyboard:       Res<Input<KeyCode>>,
) {

    let speed = 10.0;

    if keyboard.pressed(KeyCode::W) {
        camera_target.y += speed;
    }
    if keyboard.pressed(KeyCode::S) {
        camera_target.y -= speed;
    }
    if keyboard.pressed(KeyCode::A) {
        camera_target.x -= speed;
    }
    if keyboard.pressed(KeyCode::D) {
        camera_target.x += speed;
    }

    // Smooth camera.
    let blend_ratio = 0.18;
    let movement = *camera_target - *camera_current;
    *camera_current += movement * blend_ratio;

    // Update all sprite cameras.
    for mut camera_transform in query_cameras.iter_mut() {
        camera_transform.translation.x = camera_current.x;
        camera_transform.translation.y = camera_current.y;
    }
}

