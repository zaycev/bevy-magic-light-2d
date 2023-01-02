use bevy::prelude::*;

pub mod gi;
pub mod prelude;

#[derive(Component)]
pub struct SpriteCamera;
#[derive(Component)]
pub struct FloorCamera;
#[derive(Component)]
pub struct WallCamera;
#[derive(Component)]
pub struct ObjectCamera;
