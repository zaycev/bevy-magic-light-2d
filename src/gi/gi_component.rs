use bevy::prelude::*;
use bevy::prelude::{Color, Component, Vec2};
use bevy_inspector_egui::Inspectable;

#[derive(Reflect, Component, Clone, Copy, Default, Inspectable)]
#[reflect(Component)]
pub struct LightSource {
    pub radius: f32,
    pub intensity: f32,
    pub color: Color,
    pub falloff: Vec3,
    pub jitter_intensity: f32,
    pub jitter_translation: f32,
}

#[derive(Reflect, Component, Default, Inspectable)]
#[reflect(Component)]
pub struct LightOccluder {
    pub h_size: Vec2,
}

#[derive(Reflect, Component, Default, Inspectable)]
#[reflect(Component)]
pub struct DebugLight;

#[derive(Reflect, Component, Default, Inspectable)]
#[reflect(Component)]
pub struct AmbientMask {
    pub h_size: Vec2,
}

#[derive(Reflect, Component, Clone, Copy, Default, Inspectable)]
#[reflect(Component)]
pub struct GiAmbientLight {
    pub color: Color,
    pub intensity: f32,
}
