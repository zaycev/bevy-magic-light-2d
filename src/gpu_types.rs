use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

#[derive(ShaderType, Debug, Clone)]
pub struct GpuLightOmni
{
    pub color:     Vec3,
    pub intensity: f32,
    pub r_max:     f32,
    pub center:    Vec2,
}

#[derive(Default, Clone, Debug, ShaderType)]
pub struct GpuLightOmniBuffer
{
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuLightOmni>,
}

#[derive(Default, Clone, Debug, ShaderType)]
pub struct GpuGlobalParams
{
    pub screen_size:       Vec2,
    pub screen_size_inv:   Vec2,
    pub view_proj:         Mat4,
    pub inverse_view_proj: Mat4,
}

impl GpuGlobalParams
{
    pub fn set_camera_params(
        &mut self,
        window: &Window,
        camera: &Camera,
        transform: &GlobalTransform,
    )
    {
        let projection = camera.clip_from_view();
        let inverse_projection = projection.inverse();
        let view = transform.compute_matrix();
        let inverse_view = view.inverse();

        // TODO
        let window_size = window.size() * window.scale_factor();
        self.view_proj = projection * inverse_view;
        self.inverse_view_proj = view * inverse_projection;
        self.screen_size = window_size;
        self.screen_size_inv = Vec2::splat(1.0) / window_size;
    }
}
