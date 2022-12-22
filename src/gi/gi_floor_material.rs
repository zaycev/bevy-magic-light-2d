// use bevy::prelude::*;
// use bevy::render::render_resource::{ShaderRef, AsBindGroup};
// use bevy::sprite::Material2d;
// use bevy::reflect::TypeUuid;
//
// #[derive(AsBindGroup, TypeUuid, Debug, Clone)]
// #[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
// pub struct FloorMaterial {
//     #[uniform(0)]
//     color: Color,
//
//     #[texture(1)]
//     #[sampler(2)]
//     diffuse_tex: Handle<Image>,
//
//     #[texture(3)]
//     #[sampler(4)]
//     irradiance_tex: Handle<Image>,
// }
//
// impl Material2d for FloorMaterial {
//     fn fragment_shader() -> ShaderRef {
//         "shaders/materials/floor_material.wgsl".into()
//     }
// }