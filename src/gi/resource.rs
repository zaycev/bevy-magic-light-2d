use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[rustfmt::skip]
#[derive(Resource, Default, Inspectable, Copy, Clone)]
pub struct BevyMagicLight2DSettings {
    pub light_pass_params: LightPassParams,
}

#[rustfmt::skip]
#[derive(Reflect, Resource, Inspectable, Copy, Clone)]
pub struct LightPassParams {
    #[inspectable(min = 1, max = 64)]
    pub reservoir_size: u32,

    pub smooth_kernel_size: (u32, u32),

    #[inspectable(min = 0.0, max = 1.0)]
    pub direct_light_contrib: f32,

    #[inspectable(min = 0.0, max = 1.0)]
    pub indirect_light_contrib: f32,
}

impl Default for LightPassParams {
    fn default() -> Self {
        Self {
            reservoir_size: 8,
            smooth_kernel_size: (2, 1),
            direct_light_contrib: 0.2,
            indirect_light_contrib: 0.8,
        }
    }
}

#[rustfmt::skip]
#[derive(Default, Resource, Copy, Clone)]
pub struct ComputedTargetSizes {
    pub(crate) primary_target_size:  Vec2,
    pub(crate) primary_target_isize: IVec2,
    pub(crate) primary_target_usize: UVec2,

    pub(crate) sdf_target_size:      Vec2,
    pub(crate) sdf_target_isize:     IVec2,
    pub(crate) sdf_target_usize:     UVec2,
}
