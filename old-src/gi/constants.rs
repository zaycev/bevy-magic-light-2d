use bevy::prelude::*;

use crate::gi::compositing::PostProcessingMaterial;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

pub const POST_PROCESSING_RECT: Handle<Mesh> = Handle::weak_from_u128(23475629871623176235);
pub const POST_PROCESSING_MATERIAL: Handle<PostProcessingMaterial> =
    Handle::weak_from_u128(52374048672736472871);
