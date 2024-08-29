use bevy::prelude::*;

pub mod gi;
pub mod prelude;

#[derive(Component)]
pub struct SpriteCamera;
#[derive(Component)]
pub struct FloorCamera;
#[derive(Component)]
pub struct WallsCamera;
#[derive(Component)]
pub struct ObjectsCamera;
