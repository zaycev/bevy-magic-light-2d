use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use magic_2d::pipelines::Magic2DPipelineParams;
use magic_2d::prelude::{Magic2DPipelineBasicParams, Magic2DPlugin, Magic2DPluginConfig};

fn main()
{
    let clear_color = ClearColor(Color::srgba_u8(255, 255, 255, 0));

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
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
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

fn on_setup(_cmds: Commands) {}

fn on_update(_cmds: Commands) {}
