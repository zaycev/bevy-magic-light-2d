use bevy::prelude::*;

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

impl From<(f32, f32)> for LightOccluder2D
{
    fn from(value: (f32, f32)) -> Self
    {
        LightOccluder2D {
            h_size: value.into(),
        }
    }
}

impl From<Vec2> for LightOccluder2D
{
    fn from(value: Vec2) -> Self
    {
        LightOccluder2D { h_size: value }
    }
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
