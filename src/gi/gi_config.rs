use bevy::prelude::{HandleUntyped, Shader};
use bevy::reflect::TypeUuid;

pub const GI_SCREEN_PROBE_SIZE:         i32 = 8;
pub const GI_SDF_MAX_STEPS:             i32 = 16;
pub const GI_SDF_JITTER_CONTRIB:        f32 = 0.5;

pub const SHADER_GI_CAMERA:         HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1371231089456109822);
pub const SHADER_GI_TYPES:          HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4462033275253590181);
pub const SHADER_GI_ATTENUATION:    HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5254739165481917368);
pub const SHADER_GI_HALTON:         HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1287391288877821366);
pub const SHADER_GI_MATH:           HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2387462894328787238);
