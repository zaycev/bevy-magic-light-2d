use bevy::prelude::*;

use crate::gpu_types::GpuLightOmni;

// Default parameters for the light attenuation function:
// F(r) = (e^(-9.0*r^2 / r_max^2) - e^(-9.0) ) / (1.0 - e^(-9.0))
// See:
// 1) Fundamentals of Game Engine Rendering, Vol 2, Page 145
// 2) https://www.desmos.com/calculator/ilwph31m0q
const DEFAULT_R_MAX: f32 = 100.0;

/// Basic omni directional light.
///
#[derive(Debug, Default, Clone, Component)]
pub struct LightOmni
{
    pub color: LinearRgba,
    pub r_max: f32,
}

impl LightOmni
{
    pub const WHITE: Self = Self {
        color: LinearRgba::WHITE,
        r_max: DEFAULT_R_MAX,
    };

    pub fn as_gpu(&self, xform: &GlobalTransform) -> GpuLightOmni
    {
        GpuLightOmni {
            color:     Vec3::new(self.color.red, self.color.green, self.color.blue),
            intensity: self.color.alpha,
            r_max:     self.r_max,
            center:    xform.translation().truncate(),
        }
    }
}

/// TODO: implement this
///
#[derive(Debug, Default, Clone, Component)]
pub struct LightPrecomputed {}

/// TODO: implement this
///
#[derive(Debug, Default, Clone, Component)]
pub struct LightDirectional {}

/// Basic rectangular occluder.
///
#[derive(Debug, Default, Clone, Component)]
pub struct OccluderBlock
{
    pub half_size: Vec2,
}

impl OccluderBlock
{
    pub fn new(half_size: Vec2) -> Self
    {
        Self { half_size }
    }

    pub fn with_size(size: Vec2) -> Self
    {
        Self {
            half_size: size * 0.5,
        }
    }

    pub fn splat_size(size: f32) -> Self
    {
        Self {
            half_size: Vec2::splat(size * 0.5),
        }
    }

    pub fn splat_half_size(half_size: f32) -> Self
    {
        Self {
            half_size: Vec2::splat(half_size),
        }
    }
}

/// TODO: implement this
///
#[derive(Debug, Default, Clone, Component)]
pub struct OccluderSprite {}

/// TODO: implement this
///
#[derive(Debug, Default, Clone, Component)]
pub struct Occluder {}

/// TODO: implement this
///
#[derive(Debug, Default, Clone, Component)]
pub struct LightBlockDirectionalMask {}
