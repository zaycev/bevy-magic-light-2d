use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::texture::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_magic_light_2d::prelude::*;
use bevy_ecs_tilemap::*;
use helpers::ldtk::LdtkPlugin;
use rand::prelude::*;

mod helpers;

pub const SCREEN_SIZE: (f32, f32) = (1280.0, 720.0);
pub const CAMERA_SCALE: f32 = 0.6;

// Misc components.
#[derive(Component)]
pub struct MouseLight;
#[derive(Component)]
pub struct Movable;

fn main()
{
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::rgba_u8(0, 0, 0, 0)))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: SCREEN_SIZE.into(),
                        title: "Bevy Magic Light 2D: Tilemap Example".into(),
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
            TilemapPlugin,
            LdtkPlugin,
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
        .add_systems(Startup, setup.after(setup_post_processing_camera))
        .add_systems(Update, system_move_camera)
        .add_systems(Update, system_control_mouse_light.after(system_move_camera))
        .run();
}

#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
fn setup(
    mut commands: Commands,
    camera_targets: Res<CameraTargets>,
    asset_server: Res<AssetServer>,
)
{

    commands.spawn((
        SpatialBundle::default(),
        OmniLightSource2D{
            intensity: 0.5,
            color: Color::rgb_u8(255, 255, 255),
            falloff: Vec3::new(25.0, 15.0, 0.14),
            ..default()
        },
    ));

    // Add skylight light.
    commands.spawn((
        SkylightLight2D {
            color:     Color::rgb_u8(128, 158, 179),
            intensity: 0.003,
        },
        Name::new("global_skylight"),
    ));

    // Add light source.
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("cursor_light"))
        .insert(OmniLightSource2D {
            intensity: 1.0,
            color: Color::rgb_u8(254, 100, 34),
            falloff: Vec3::new(25.0, 35.0, 0.5),
            ..default()
        })
        .insert(RenderLayers::all())
        .insert(MouseLight);

    let projection = OrthographicProjection {
        scale: CAMERA_SCALE,
        near: -2000.0,
        far: 2000.0,
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


    let handle: Handle<helpers::ldtk::LdtkMap> = asset_server.load("map.ldtk");

    commands.spawn(helpers::ldtk::LdtkMapBundle {
        ldtk_map: handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

}

fn system_control_mouse_light(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut query_light: Query<(&mut Transform, &mut OmniLightSource2D), With<MouseLight>>,
    query_cameras: Query<(&Camera, &GlobalTransform), With<SpriteCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
)
{
    let mut rng = thread_rng();

    // We only need to iter over first camera matched.
    let (camera, camera_transform) = query_cameras.iter().next().unwrap();
    let Ok(window) = window.get_single() else {
        return;
    };

    if let Some(screen_pos) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let mut mouse_ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        mouse_ndc = Vec2::new(mouse_ndc.x, -mouse_ndc.y);
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let mouse_world = ndc_to_world.project_point3(mouse_ndc.extend(-1.0));

        let (mut mouse_transform, mut mouse_color) = query_light.single_mut();
        mouse_transform.translation = mouse_world.truncate().extend(1000.0);

        if mouse.just_pressed(MouseButton::Right) {
            mouse_color.color = Color::rgba(rng.gen(), rng.gen(), rng.gen(), 1.0);
        }
        if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
            commands
                .spawn(SpatialBundle {
                    transform: Transform {
                        translation: mouse_world.truncate().extend(0.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Name::new("point_light"))
                .insert(RenderLayers::all())
                .insert(OmniLightSource2D {
                    jitter_intensity: 0.0,
                    jitter_translation: 0.0,
                    ..*mouse_color
                });
        }

    }
}

#[rustfmt::skip]
fn system_move_camera(
    mut camera_current: Local<Vec2>,
    mut camera_target:  Local<Vec2>,
    mut query_cameras:  Query<(&mut Transform, &mut OrthographicProjection), With<SpriteCamera>>,
        keyboard:       Res<ButtonInput<KeyCode>>,
) {
    let speed = 6.0;
    let mut zoom = 0.0;

    if keyboard.pressed(KeyCode::KeyW) { camera_target.y += speed; }
    if keyboard.pressed(KeyCode::KeyS) { camera_target.y -= speed; }
    if keyboard.pressed(KeyCode::KeyA) { camera_target.x -= speed; }
    if keyboard.pressed(KeyCode::KeyD) { camera_target.x += speed; }
    if keyboard.pressed(KeyCode::KeyQ) { zoom += 0.005; }
    if keyboard.pressed(KeyCode::KeyE) { zoom -= 0.005; }

    // Smooth camera.
    let blend_ratio = 0.2;
    let movement = *camera_target - *camera_current;
    *camera_current += movement * blend_ratio;

    // Update all sprite cameras.
    for (mut camera_transform, mut projection) in query_cameras.iter_mut() {
        camera_transform.translation.x = camera_current.x;
        camera_transform.translation.y = camera_current.y;
        projection.scale += zoom;
    }
}
