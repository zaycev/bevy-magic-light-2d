use bevy::prelude::{Gizmos, *};
use bevy::text::BreakLineOn;

use crate::components::{LightOmni, OccluderBlock};
use crate::hud::consts::HUD_GIZMO_COLOR;

#[cfg(feature = "debug_hud")]
pub mod consts
{
    use bevy::color::palettes::tailwind::*;
    use bevy::prelude::*;
    pub const HUD_TEXT_ALPHA: f32 = 0.22;
    pub const HUD_TEXT_COLOR: Srgba = EMERALD_400;
    pub const HUD_GIZMO_COLOR: Srgba = EMERALD_400;
    pub const HUD_TEXT_SIZE: f32 = 20.0;
}

#[cfg(feature = "debug_hud")]
pub fn hud_setup() {}

#[cfg(feature = "debug_hud")]
#[derive(Debug, Default, Clone, Component)]
pub struct Magic2DHudMarker;

#[cfg(feature = "debug_hud")]
#[derive(Debug, Default, Clone, Component)]
pub struct Magic2DHudText;

#[cfg(feature = "debug_hud")]
fn util_make_entity_text<T>(_: T, e: Entity, font: &Handle<Font>) -> Text2dBundle
{
    use std::any::type_name;

    use consts::*;
    let name = type_name::<T>()
        .rsplit("::")
        .next()
        .expect("type_name split failed");
    let color: Color = HUD_TEXT_COLOR.into();
    let color = color.with_alpha(HUD_TEXT_ALPHA);

    Text2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.1),
        text: Text {
            justify:            JustifyText::Center,
            linebreak_behavior: BreakLineOn::NoWrap,
            sections:           vec![
                TextSection {
                    value: format!("{}#{}\n", name, e),
                    style: TextStyle {
                        color,
                        font: font.clone(),
                        font_size: HUD_TEXT_SIZE,
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        color,
                        font: font.clone(),
                        font_size: HUD_TEXT_SIZE * 0.8,
                    },
                },
            ],
        },
        ..default()
    }
}

#[cfg(feature = "debug_hud")]
pub fn hud_update(
    mut cmds: Commands,
    mut gizmos: Gizmos,

    queries_new: (
        Query<(Entity, &LightOmni), Without<Magic2DHudMarker>>,
        Query<(Entity, &OccluderBlock), Without<Magic2DHudMarker>>,
    ),

    query_existing: (
        Query<(&mut Text, &GlobalTransform), With<Magic2DHudText>>,
        Query<(&LightOmni, &GlobalTransform), With<Magic2DHudMarker>>,
        Query<(&OccluderBlock, &GlobalTransform), With<Magic2DHudMarker>>,
    ),

    asset_loader: Res<AssetServer>,
)
{
    let font = asset_loader.load("font/JetBrainsMono-Light.ttf");

    {
        let (light_omni, occluder_block) = queries_new;
        for (e, v) in light_omni.iter() {
            cmds.entity(e).insert(Magic2DHudMarker).with_children(|p| {
                p.spawn((Magic2DHudText, util_make_entity_text(v, e, &font)));
            });
        }
        for (e, v) in occluder_block.iter() {
            cmds.entity(e).insert(Magic2DHudMarker).with_children(|p| {
                p.spawn((Magic2DHudText, util_make_entity_text(v, e, &font)));
            });
        }
    }

    {
        let (mut texts, light_omni, occluder_block) = query_existing;
        let color: Color = HUD_GIZMO_COLOR.into();
        let color = color.with_alpha(0.35);
        for (light, xform) in &light_omni {
            gizmos.circle_2d(xform.translation().truncate(), light.r_max, color);
        }
        for (occluder, xform) in &occluder_block {
            gizmos.rect_2d(
                xform.translation().truncate(),
                0.0,
                occluder.half_size * 2.0,
                color,
            );
        }
        for (mut t, xform) in &mut texts {
            let xy = xform.translation().truncate();
            t.sections[1].value = format!("[{:.2}:{:.2}]", xy.x, xy.y)
        }
    }
}
