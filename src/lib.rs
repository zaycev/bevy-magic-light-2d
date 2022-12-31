use bevy::prelude::*;

pub mod gi;

#[derive(Component)]
pub struct MainCamera;

pub const SCREEN_SIZE: (usize, usize) = (1024, 1024);
