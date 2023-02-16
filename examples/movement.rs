use std::f64::consts::PI;

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
// use bevy_inspector_egui::prelude::*;
use bevy_magic_light_2d::prelude::*;

#[derive(Debug, Component)]
struct Mover;

fn main() {
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1024., 1024.).into(),
                title: "Bevy Magic Light 2D: Square Example".into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(BevyMagicLight2DPlugin)
        // .add_plugin(InspectorPlugin::<BevyMagicLight2DSettings>::new())
        .add_startup_system(setup.after(setup_post_processing_camera))
        .add_system(system_move_camera)
        .add_system(move_collider)
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 8,
                smooth_kernel_size: (3, 3),
                direct_light_contrib: 0.5,
                indirect_light_contrib: 0.5,
                ..default()
            },
        })
        .run();
}

fn setup(mut commands: Commands, post_processing_target: Res<PostProcessingTarget>) {
    let mut occluders = vec![];
    let occluder_entity = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            GlobalTransform::default(),
            Visibility::Visible,
            ComputedVisibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(80.0, 40.0),
            },
            Mover,
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
        let spawn_light = |cmd: &mut Commands,
                           x: f32,
                           y: f32,
                           name: &'static str,
                           light_source: OmniLightSource2D| {
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

        lights.push(spawn_light(
            &mut commands,
            -512.,
            -512.,
            "left",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(255, 255, 0),
                falloff: Vec3::new(1.5, 10.0, 0.01),
                ..default()
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            512.,
            -512.,
            "right",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(0, 255, 255),
                falloff: Vec3::new(1.5, 10.0, 0.01),
                ..default()
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
                    target: RenderTarget::Image(render_target),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("main_camera"),
        ))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
}

fn system_move_camera(
    mut camera_target: Local<Vec3>,
    mut query_camera: Query<&mut Transform>,
    keyboard: Res<Input<KeyCode>>,
) {
    if let Ok(mut camera_transform) = query_camera.get_single_mut() {
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
        let movement = (*camera_target - camera_transform.translation) * blend_ratio;
        camera_transform.translation.x += movement.x;
        camera_transform.translation.y += movement.y;
    }
}

fn move_collider(mut query_mover: Query<&mut Transform, With<Mover>>, time: Res<Time>) {
    let radius = 100.;
    let cycle_secs = 5.;
    let elapsed = time.elapsed().as_secs_f64();
    let curr_time = elapsed % cycle_secs;
    let theta = (curr_time / cycle_secs) * 2. * PI;

    if let Ok(mut transform) = query_mover.get_single_mut() {
        transform.translation.x = radius * theta.cos() as f32;
        transform.translation.y = radius * theta.sin() as f32;
        transform.rotation = Quat::from_rotation_z(theta as f32);
    }
}
