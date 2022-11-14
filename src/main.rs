use bevy::{prelude::*, render::camera::RenderTarget, sprite::MaterialMesh2dBundle, core_pipeline::bloom::BloomSettings};
use bevy_2d_gi_experiment::{
    gi::{self, LightOccluder, LightSource, GiTarget},
    MainCamera, SCREEN_SIZE,
};
use rand::prelude::*;

const MAP: &[&[u8]] = &[
    &[1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0 , 0, 1, 0],
    &[0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 1, 0],
    &[0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 1, 0],
    &[0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0 , 0, 1, 0],
    &[1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0 , 0, 0, 0],
    &[1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 1, 0],
    &[0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0 , 0, 1, 0],
    &[0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 1, 0],
    &[1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0 , 0, 1, 0],
    &[1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0 , 0, 0, 0],
    &[0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0 , 0, 1, 0],
];

#[derive(Component)]
pub struct MouseLight;

fn main() {
    App::new()
        // .insert_resource(WindowDescriptor {
        //     width: SCREEN_SIZE.0 as f32,
        //     height: SCREEN_SIZE.1 as f32,
        //     title: "Bevy 2D GI Experiment".into(),
        //     resizable: false,
        //     mode: bevy::window::WindowMode::Windowed,
        //     ..Default::default()
        // })
        .insert_resource(ClearColor(Color::rgba(0.0, 0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }).set(WindowPlugin{
            window: WindowDescriptor {
                width: SCREEN_SIZE.0 as f32,
                height: SCREEN_SIZE.1 as f32,
                title: "Bevy 2D GI Experiment".into(),
                resizable: false,
                mode: bevy::window::WindowMode::Windowed,
                ..Default::default()
             },
             ..Default::default()
        }))
        .add_plugin(gi::GiComputePlugin)
        .add_startup_system(setup)
        .add_system(system_move_light_to_cursor)
        .add_system(system_move_camera.before(system_move_light_to_cursor))
        .add_system(system_move_target.after(system_move_camera))
        .run();
}

fn setup(
    mut commands:     Commands,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<ColorMaterial>>,
) {
    let rows = MAP.len();
    let cols = MAP[0].len();
    let block_size = Vec2::new((SCREEN_SIZE.0 / cols) as f32, (SCREEN_SIZE.1 / rows) as f32);

    let center_offset = Vec2::new(-(SCREEN_SIZE.0 as f32) / 2.0, (SCREEN_SIZE.1 as f32) / 2.0)
        + block_size / 2.0
        - Vec2::new(0.0, block_size.y);

    let block_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let block_material = materials.add(ColorMaterial::from(Color::DARK_GRAY));

    // Add light occluders from MAP.
    for (i, row) in MAP.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if *cell == 1 {
                let translation = center_offset
                    + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y);

                commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: block_mesh.clone().into(),
                        material: block_material.clone(),
                        transform: Transform {
                            translation: Vec3::new(translation.x, translation.y, 0.0),
                            scale: block_size.extend(0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(LightOccluder {
                        h_size: block_size / 2.0,
                    });
            }
        }
    }

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
        .insert(LightSource {
            intensity: 1.0,
            radius: 32.0,
            color: Color::rgba(0.54057753, 0.3112858, 0.096047044, 1.0),
        })
        .insert(MouseLight);

    commands
        .spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                priority: 1,
                ..Default::default()
            },
            ..Default::default()
        }, BloomSettings::default()))
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
                if mouse.just_pressed(MouseButton::Left) {
                    commands
                        .spawn(SpatialBundle {
                            transform: Transform {
                                translation: mouse_world.truncate().extend(100.0),
                                scale: Vec3::splat(8.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(LightSource {
                            intensity: 1.0,
                            radius:    32.0,
                            color:     mouse_color.color,
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
            target_transform.translation = camera_transform.translation;
        }
    }
}