use crate::gi::WORKGROUP_SIZE;
use bevy::asset::AssetPath;
use bevy::math::IVec2;

pub struct AssetUtil;

impl AssetUtil {
    pub fn gi(l: &'static str) -> AssetPath {
        AssetPath::new("gi/target".into(), Some(l.to_owned()))
    }
    pub fn camera(l: &'static str) -> AssetPath {
        AssetPath::new("camera/target".into(), Some(l.to_owned()))
    }
    pub fn material(l: &'static str) -> AssetPath {
        AssetPath::new("material".into(), Some(l.to_owned()))
    }
    pub fn mesh(l: &'static str) -> AssetPath {
        AssetPath::new("mesh".into(), Some(l.to_owned()))
    }
}

pub fn align_to_work_group_grid(size: IVec2) -> IVec2 {
    let wg_size = WORKGROUP_SIZE as i32;
    size + IVec2::new(wg_size - size.x % wg_size, wg_size - size.y % wg_size)
}
