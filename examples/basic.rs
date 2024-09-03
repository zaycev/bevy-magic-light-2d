use bevy::color::palettes::basic::{PURPLE, YELLOW};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PrimaryWindow;
use magic_2d::components::{LightOmni, OccluderBlock};
use magic_2d::pipelines::Magic2DPipelineParams;
use magic_2d::prelude::{Magic2DPipelineBasicParams, Magic2DPlugin, Magic2DPluginConfig};

fn main()
{
    let clear_color = ClearColor(Color::srgba_u8(0, 0, 0, 0));

    App::new()
        .insert_resource(clear_color)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1280., 720.).into(),
                    title: "Magic Light 2D: Minimal".into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
            Magic2DPlugin {
                config: Magic2DPluginConfig {
                    pipeline: Magic2DPipelineParams::Basic(Magic2DPipelineBasicParams {}),
                },
            },
        ))
        .add_systems(Startup, on_setup)
        .add_systems(Update, on_update)
        .run();
}

#[derive(Component)]
pub struct FollowCursor;

fn on_setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
)
{
    // Create 2D occluder.
    let occluder_half_size = 32.0;

    let mesh_rect = meshes.add(Rectangle {
        half_size: Vec2::splat(occluder_half_size),
        ..Default::default()
    });
    let mesh_circle = meshes.add(Circle { radius: 5.0 });

    cmds.spawn((
        MaterialMesh2dBundle {
            mesh: mesh_rect.clone().into(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: materials.add(Color::from(PURPLE)),
            ..default()
        },
        OccluderBlock::splat_half_size(occluder_half_size),
    ));

    // Create light source.
    cmds.spawn((
        MaterialMesh2dBundle {
            mesh: mesh_circle.clone().into(),
            transform: Transform::from_xyz(100.0, 100.0, 0.0),
            material: materials.add(Color::from(YELLOW)),
            ..default()
        },
        LightOmni {
            color: LinearRgba::new(1.0, 1.0, 1.0, 0.5),
            r_max: 300.0,
        },
        FollowCursor,
    ));

    // Create 2D camera.
    cmds.spawn(Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn on_update(
    mut query_lights: Query<&mut Transform, With<FollowCursor>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform)>,
)
{
    let window = query_window.single();
    let (camera, camera_transform) = query_camera.single();
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let window_size = window.size();
    let ndc = (cursor / window_size) * 2.0 - Vec2::ONE;
    let proj = camera_transform.compute_matrix() * camera.clip_from_view().inverse();
    let world = proj
        .project_point3(Vec3::new(ndc.x, -ndc.y, 1.0))
        .truncate()
        .extend(0.0);

    for mut transform in query_lights.iter_mut() {
        transform.translation = world;
    }
}
