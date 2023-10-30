use bevy::asset::AssetPath;

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
