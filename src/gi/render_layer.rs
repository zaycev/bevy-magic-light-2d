use bevy::render::view::Layer;

pub const LAYER_FLOOR_ID: Layer = 1;
pub const LAYER_WALLS_ID: Layer = 2;
pub const LAYER_OBJECTS_ID: Layer = 3;

pub const CAMERA_LAYER_FLOOR: &[Layer] = &[LAYER_FLOOR_ID];
pub const CAMERA_LAYER_WALLS: &[Layer] = &[LAYER_WALLS_ID];
pub const CAMERA_LAYER_OBJECTS: &[Layer] = &[LAYER_OBJECTS_ID];

pub const ALL_LAYERS: &[Layer] = &[LAYER_FLOOR_ID, LAYER_WALLS_ID, LAYER_OBJECTS_ID];
