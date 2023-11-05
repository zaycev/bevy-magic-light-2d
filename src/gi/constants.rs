use bevy::prelude::*;

use crate::gi::compositing::PostProcessingMaterial;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

pub const SHADER_GI_CAMERA: Handle<Shader> = Handle::weak_from_u128(1371231089456109822);
pub const SHADER_GI_TYPES: Handle<Shader> = Handle::weak_from_u128(4462033275253590181);
pub const SHADER_GI_ATTENUATION: Handle<Shader> = Handle::weak_from_u128(5254739165481917368);
pub const SHADER_GI_HALTON: Handle<Shader> = Handle::weak_from_u128(1287391288877821366);
pub const SHADER_GI_MATH: Handle<Shader> = Handle::weak_from_u128(2387462894328787238);
pub const SHADER_GI_RAYMARCH: Handle<Shader> = Handle::weak_from_u128(9876835068496322894);

pub const POST_PROCESSING_QUAD: Handle<Mesh> = Handle::weak_from_u128(23475629871623176235);
pub const POST_PROCESSING_MATERIAL: Handle<PostProcessingMaterial> =
    Handle::weak_from_u128(52374048672736472871);
