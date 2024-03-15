use std::collections::HashMap;
use std::io::ErrorKind;

use bevy::asset::{AssetLoader, AssetPath, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::view::RenderLayers;
use bevy::utils::BoxedFuture;
use bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform;
use bevy_ecs_tilemap::map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::TilemapBundle;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex};
use serde_json::Value;
use thiserror::Error;

use bevy_magic_light_2d::prelude::*;

#[derive(Default)]
pub struct LdtkPlugin;

impl Plugin for LdtkPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_asset::<LdtkMap>()
            .register_asset_loader(LdtkLoader)
            .add_systems(Update, process_loaded_tile_maps);
    }
}

#[derive(TypePath, Asset)]
pub struct LdtkMap
{
    pub project:  ldtk_rust::Project,
    pub tilesets: HashMap<i64, Handle<Image>>,
}

#[derive(Default, Component)]
pub struct LdtkMapConfig
{
    pub selected_level: usize,
}

#[derive(Default, Bundle)]
pub struct LdtkMapBundle
{
    pub ldtk_map:         Handle<LdtkMap>,
    pub ldtk_map_config:  LdtkMapConfig,
    pub transform:        Transform,
    pub global_transform: GlobalTransform,
}

pub struct LdtkLoader;

#[derive(Component)]
pub struct Object;

#[derive(Debug, Error)]
pub enum LdtkAssetLoaderError
{
    /// An [IO](std::io) Error
    #[error("Could not load LDTk file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for LdtkLoader
{
    type Asset = LdtkMap;
    type Settings = ();
    type Error = LdtkAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<LdtkMap, Self::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let project: ldtk_rust::Project = serde_json::from_slice(&bytes).map_err(|e| {
                std::io::Error::new(
                    ErrorKind::Other,
                    format!("Could not read contents of Ldtk map: {e}"),
                )
            })?;
            let dependencies: Vec<(i64, AssetPath)> = project
                .defs
                .tilesets
                .iter()
                .filter_map(|tileset| {
                    tileset.rel_path.as_ref().map(|rel_path| {
                        (
                            tileset.uid,
                            load_context.path().parent().unwrap().join(rel_path).into(),
                        )
                    })
                })
                .collect();

            let ldtk_map = LdtkMap {
                project,
                tilesets: dependencies
                    .iter()
                    .map(|dep| (dep.0, load_context.load(dep.1.clone())))
                    .collect(),
            };
            Ok(ldtk_map)
        })
    }

    fn extensions(&self) -> &[&str]
    {
        static EXTENSIONS: &[&str] = &["ldtk"];
        EXTENSIONS
    }
}

pub fn process_loaded_tile_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<LdtkMap>>,
    maps: Res<Assets<LdtkMap>>,
    mut query: Query<(Entity, &Handle<LdtkMap>, &LdtkMapConfig)>,
    new_maps: Query<&Handle<LdtkMap>, Added<Handle<LdtkMap>>>,
    mut map_objects: Query<Entity, With<Object>>,
)
{
    let mut changed_maps = Vec::<AssetId<LdtkMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                log::info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                log::info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities, add them to the changed_maps list
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.id());
    }

    for changed_map in changed_maps.iter() {

        for entity in map_objects.iter() {
            commands.entity(entity).despawn();
        }


        for (entity, map_handle, map_config) in query.iter_mut() {
            // only deal with currently changed map
            if map_handle.id() != *changed_map {
                continue;
            }
            if let Some(ldtk_map) = maps.get(map_handle) {
                // Despawn all existing tilemaps for this LdtkMap
                commands.entity(entity).despawn_descendants();

                // Pull out tilesets and their definitions into a new hashmap
                let mut tilesets = HashMap::new();
                ldtk_map.project.defs.tilesets.iter().for_each(|tileset| {
                    tilesets.insert(
                        tileset.uid,
                        (
                            ldtk_map.tilesets.get(&tileset.uid).unwrap().clone(),
                            tileset,
                        ),
                    );
                });

                let default_grid_size = ldtk_map.project.default_grid_size;
                let level = &ldtk_map.project.levels[map_config.selected_level];

                let map_tile_count_x = (level.px_wid / default_grid_size) as u32;
                let map_tile_count_y = (level.px_hei / default_grid_size) as u32;

                let size = TilemapSize {
                    x: map_tile_count_x,
                    y: map_tile_count_y,
                };

                // We will create a tilemap for each layer in the following loop
                for (layer_id, layer) in level
                    .layer_instances
                    .as_ref()
                    .unwrap()
                    .iter()
                    .rev()
                    .enumerate()
                {
                    let width_px = level.px_wid as f32;
                    let height_px = level.px_hei as f32;

                    for instance in layer.entity_instances.iter() {
                        let at = instance.px[0] as f32 - width_px * 0.5;
                        let at_y = height_px - instance.px[1] as f32 - height_px * 0.5;
                        let tile = instance
                            .field_instances
                            .iter()
                            .find_map(|field| {
                                if field.identifier == "Tile" {
                                    if let Some(tile) = &field.tile
                                    {
                                        return Some(tile.clone())
                                    }
                                }
                                None
                            })
                            .unwrap();
                        let is_occluder = instance
                            .field_instances
                            .iter()
                            .find_map(|field| {
                                if field.identifier == "OccludeLight" {
                                    if let Some(val) = &field.value
                                    {
                                        return Some(val.as_bool().unwrap())
                                    }
                                }
                                None
                            })
                            .unwrap_or(false);
                        let is_light = instance
                            .field_instances
                            .iter()
                            .find_map(|field| {
                                if field.identifier == "EmitLight" {
                                    if let Some(val) = &field.value
                                    {
                                        return Some(val.as_bool().unwrap())
                                    }
                                }
                                None
                            })
                            .unwrap_or(false);

                        let color = instance
                            .field_instances
                            .iter()
                            .find_map(|field| {
                                if field.identifier == "Color" {
                                    return Some(Color::hex(field.value.clone().unwrap().as_str().unwrap()).unwrap());
                                }
                                None
                            }).unwrap_or(Color::BLACK);


                        let uid = tile.tileset_uid;
                        let (texture, _) = tilesets.get(&uid).unwrap().clone();
                        let mut transform = Transform::from_xyz(at, at_y, 100.0);

                        let entity = commands.spawn((
                            Object,
                            SpriteBundle {
                                sprite: Sprite {
                                    rect: Some(Rect {
                                        min: Vec2::new(tile.x as f32, tile.y as f32),
                                        max: Vec2::new((tile.x + tile.w) as f32, (tile.y + tile.h) as f32),
                                    }),
                                    ..default()
                                },
                                transform,
                                texture: texture.clone(),
                                ..default()
                            },
                            RenderLayers::from_layers(CAMERA_LAYER_OBJECTS),
                        )).id();

                        if is_occluder {
                            commands.entity(entity).insert(LightOccluder2D {
                                h_size: Vec2::splat(8.0),
                            });
                        }

                        if is_light {
                            commands.entity(entity).insert(OmniLightSource2D{
                                intensity: 7.0,
                                color,
                                falloff: Vec3::new(25.0, 15.0, 0.5),
                                jitter_intensity: 0.2,
                                jitter_translation: 4.0,
                                ..default()
                            });
                        }

                    }

                    if let Some(uid) = layer.tileset_def_uid {
                        let (texture, tileset) = tilesets.get(&uid).unwrap().clone();

                        // Tileset-specific tilemap settings
                        let tile_size = TilemapTileSize {
                            x: tileset.tile_grid_size as f32,
                            y: tileset.tile_grid_size as f32,
                        };

                        // Pre-emptively create a map entity for tile creation
                        let map_entity = commands.spawn_empty().id();
                        let grid_size = tile_size.into();
                        let map_type = TilemapType::default();
                        let center = get_tilemap_center_transform(
                            &size,
                            &grid_size,
                            &map_type,
                            layer_id as f32,
                        );

                        // Create tiles for this layer from LDtk's grid_tiles and auto_layer_tiles
                        let mut floor_storage = TileStorage::empty(size);
                        let mut wall_storage = TileStorage::empty(size);

                        for (idx, tile) in layer
                            .grid_tiles
                            .iter()
                            .chain(layer.auto_layer_tiles.iter())
                            .enumerate()
                        {
                            let val = layer.int_grid_csv[idx];
                            let is_wall = val == 2;

                            let mut position = TilePos {
                                x: (tile.px[0] / default_grid_size) as u32,
                                y: (tile.px[1] / default_grid_size) as u32,
                            };

                            position.y = map_tile_count_y - position.y - 1;

                            let tile_entity = commands
                                .spawn((
                                    TileBundle {
                                        position,
                                        tilemap_id: TilemapId(map_entity),
                                        texture_index: TileTextureIndex(tile.t as u32),
                                        ..default()
                                    },
                                    RenderLayers::from_layers(CAMERA_LAYER_FLOOR),
                                ))
                                .id();

                            if is_wall {
                                let at_x = ((idx as u32 % map_tile_count_x)
                                    * default_grid_size as u32)
                                    as f32
                                    - width_px * 0.5
                                    + 8.0;
                                let at_y = (height_px
                                    - ((idx as u32 / map_tile_count_x) * default_grid_size as u32)
                                        as f32)
                                    - height_px * 0.5
                                    - 8.0;

                                commands
                                    .entity(tile_entity)
                                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                                    .insert(SpatialBundle::from_transform(Transform::from_xyz(
                                        at_x, at_y, 0.0,
                                    )))
                                    .insert(LightOccluder2D {
                                        h_size: Vec2::splat(8.0),
                                    });

                                wall_storage.set(&position, tile_entity);
                            } else {
                                floor_storage.set(&position, tile_entity);
                            }
                        }

                        commands
                            .entity(map_entity)
                            .insert(TilemapBundle {
                                grid_size,
                                map_type,
                                size,
                                storage: floor_storage,
                                texture: TilemapTexture::Single(texture.clone()),
                                tile_size,
                                transform: center,
                                ..default()
                            })
                            .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR));

                        commands
                            .entity(map_entity)
                            .insert(TilemapBundle {
                                grid_size,
                                map_type,
                                size,
                                storage: wall_storage,
                                texture: TilemapTexture::Single(texture),
                                tile_size,
                                transform: center,
                                ..default()
                            })
                            .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS));
                    }
                }
            }
        }
    }
}
