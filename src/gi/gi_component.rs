use bevy::prelude::{Component, Vec2, Color};
use bevy_inspector_egui::Inspectable;

#[derive(Component, Inspectable, Clone, Copy)]
pub struct LightSource {
    pub radius:    f32,
    pub intensity: f32,
    pub color:     Color,
}

#[derive(Component)]
pub struct LightOccluder {
    pub h_size: Vec2,
}

#[derive(Component)]
pub struct DebugLight;

