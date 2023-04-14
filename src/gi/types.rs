use bevy::prelude::{Color, Component, Vec2, *};

#[rustfmt::skip]
#[derive(Reflect, Component, Clone, Copy, Default)]
#[reflect(Component)]
pub struct OmniLightSource2D {
    pub intensity:          f32,
    pub color:              Color,
    pub falloff:            Vec3,
    pub jitter_intensity:   f32,
    pub jitter_translation: f32,
}

#[rustfmt::skip]
#[derive(Reflect, Component, Default, Clone, Copy)]
#[reflect(Component)]
pub struct LightOccluder2D {
    pub h_size: Vec2,
}
///
///
///
#[rustfmt::skip]
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct SkylightMask2D {
    pub h_size: Vec2,
}
///
///
///
#[rustfmt::skip]
#[derive(Reflect, Component, Clone, Copy, Default)]
#[reflect(Component)]
pub struct SkylightLight2D {
    pub color:     Color,
    pub intensity: f32,
}
