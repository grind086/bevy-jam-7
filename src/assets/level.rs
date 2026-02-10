use bevy::{
    asset::{AssetLoader, LoadContext, LoadDirectError, io::Reader},
    platform::collections::{HashMap, hash_map::Entry},
    prelude::*,
    sprite_render::{TileData, TilemapChunkTileData},
};
use thiserror::Error;

use crate::assets::{
    level::{
        level_collision::{LevelCollider, LevelCollisionBuilder},
        tileset_image::{AddTileError, TilesetImageBuilder, UnsupportedFormatError},
    },
    serialize::ldtk::{
        EntityInstance as LdtkEntity, LayerInstance as LdtkLayer, Level as LdtkLevel,
    },
};

mod level_collision;
mod tileset_image;

#[derive(Asset, Reflect)]
pub struct Level {
    pub name: String,
    pub grid_size: UVec2,
    pub grid_offset: IVec2,
    pub player_spawn: IVec2,
    pub terrain_tileset: Handle<Image>,
    pub terrain_tiledata: TilemapChunkTileData,
    pub terrain_colliders: Vec<LevelCollider>,
}

impl Level {
    pub fn bounds(&self) -> IRect {
        IRect {
            min: self.grid_offset,
            max: self.grid_offset + self.grid_size.as_ivec2(),
        }
    }

    pub fn center_position(&self) -> Vec2 {
        self.bounds().as_rect().center()
    }
}

#[derive(TypePath, Default)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
    type Asset = Level;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        &(): &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let ldtk: LdtkLevel = serde_json::from_slice(&bytes)?;
        let level_offset = IVec2::new(ldtk.world_x as _, -ldtk.world_y as _);

        let entities_layer = get_named_layer(&ldtk, "Entities").unwrap();

        let player_spawn_entity = get_named_entity(entities_layer, "Player_Spawn").unwrap();
        let player_spawn = IVec2::new(
            player_spawn_entity.grid[0] as _,
            (entities_layer.c_hei - player_spawn_entity.grid[1] - 1) as _,
        );

        let terrain_layer = get_named_layer(&ldtk, "Terrain").unwrap();

        let grid_size = UVec2::new(terrain_layer.c_wid as _, terrain_layer.c_hei as _);
        let _grid_offset = IVec2::new(
            terrain_layer.px_total_offset_x as _,
            terrain_layer.px_total_offset_y as _,
        ) / terrain_layer.grid_size as i32;

        let terrain_colliders = LevelCollisionBuilder::from_grid(
            grid_size,
            terrain_layer.int_grid_csv.iter().map(|i| *i != 0).collect(),
            true,
        )
        .build();

        let terrain_tiles_layer = get_named_layer(&ldtk, "TerrainTiles").unwrap();
        let (terrain_tileset, terrain_tiledata) =
            build_tilemap_from_layer(load_context, terrain_tiles_layer).await?;

        Ok(Level {
            name: ldtk.identifier,
            grid_size,
            grid_offset: level_offset,
            player_spawn,
            terrain_tileset,
            terrain_tiledata,
            terrain_colliders,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtkl"]
    }
}

fn get_named_layer<'a>(level: &'a LdtkLevel, name: &str) -> Option<&'a LdtkLayer> {
    level
        .layer_instances
        .as_ref()?
        .iter()
        .find(|layer| layer.identifier == name)
}

fn get_named_entity<'a>(layer: &'a LdtkLayer, name: &str) -> Option<&'a LdtkEntity> {
    layer
        .entity_instances
        .iter()
        .find(|entity| entity.identifier == name)
}

#[derive(Debug, Error)]
pub enum BuildTilemapError {
    #[error("layer has no `tileset_rel_path` property")]
    PathNotFound,
    #[error("failed to load tileset image: {0}")]
    LoadTilesetImage(#[from] LoadDirectError),
    #[error(transparent)]
    Format(#[from] UnsupportedFormatError),
    #[error("failed to copy tile from source offset {offset:?}: {error}")]
    AddTile {
        offset: UVec2,
        #[source]
        error: AddTileError,
    },
}

async fn build_tilemap_from_layer(
    load_context: &mut LoadContext<'_>,
    layer: &LdtkLayer,
) -> Result<(Handle<Image>, TilemapChunkTileData), BuildTilemapError> {
    let tileset_path = layer
        .tileset_rel_path
        .as_ref()
        .ok_or(BuildTilemapError::PathNotFound)?;
    let tileset_image = load_context
        .loader()
        .immediate()
        .load::<Image>(tileset_path)
        .await?;

    let tile_size = layer.grid_size;
    let tiles = if layer.grid_tiles.is_empty() {
        &layer.auto_layer_tiles
    } else {
        &layer.grid_tiles
    };

    let mut tile_id_map = HashMap::new();
    let mut tileset_builder = TilesetImageBuilder::new(
        UVec2::splat(tile_size as _),
        tileset_image.get().texture_descriptor.format,
    )?;

    for tile in tiles {
        let offset = UVec2::new(tile.src[0] as _, tile.src[1] as _);
        if let Entry::Vacant(e) = tile_id_map.entry(tile.t) {
            e.insert(
                tileset_builder
                    .add_tile(tileset_image.get(), offset)
                    .map_err(|error| BuildTilemapError::AddTile { offset, error })?,
            );
        }
    }

    let w = layer.c_wid as usize;
    let h = layer.c_hei as usize;
    let mut tile_data = vec![None; w * h];
    for tile in tiles {
        let i = (tile.px[0] + layer.c_wid * tile.px[1]) / tile_size;
        tile_data[i as usize] = Some(TileData::from_tileset_index(tile_id_map[&tile.t]));
    }

    // Y-flip tilemap
    for r in 0..h / 2 {
        let ptr = tile_data.as_mut_ptr();
        // SAFETY: Trust me bro. It'll be fine bro.
        unsafe { core::ptr::swap_nonoverlapping(ptr.add(r * w), ptr.add((h - r - 1) * w), w) };
    }

    let tileset_image = load_context.add_labeled_asset(
        format!("{}_tiles", layer.identifier),
        tileset_builder.build(),
    );

    Ok((tileset_image, TilemapChunkTileData(tile_data)))
}

#[cfg(feature = "dev_native")]
pub(super) mod hot_reload {
    use avian2d::prelude::RigidBody;
    use bevy::{
        asset::AssetEventSystems,
        prelude::*,
        sprite_render::{AlphaMode2d, TilemapChunk},
    };

    use crate::{
        assets::level::Level,
        demo::level::{CurrentLevel, LevelGeometry},
    };

    pub fn plugin(app: &mut App) {
        app.add_systems(
            PostUpdate,
            reload_level
                .after(AssetEventSystems)
                .run_if(on_message::<AssetEvent<Level>>),
        );
    }

    fn reload_level(
        mut asset_events: MessageReader<AssetEvent<Level>>,
        levels: Res<Assets<Level>>,
        level_handle: Single<&CurrentLevel>,
        level_geometry: Single<(Entity, &Children), With<LevelGeometry>>,
        mut commands: Commands,
    ) {
        for ev in asset_events.read() {
            match ev {
                &AssetEvent::Modified { id } if id == level_handle.id() => {
                    let level = levels.get(id).unwrap();
                    info!("Reloading level {:?}", level.name);

                    // Despawn existing tilemap and colliders
                    let despawn_batch: Vec<_> = level_geometry.1.iter().collect();

                    commands.queue(move |world: &mut World| {
                        despawn_batch.into_iter().for_each(|entity| {
                            world.despawn(entity);
                        })
                    });

                    // Spawn tilemap
                    commands.spawn((
                        Name::new("Terrain Tilemap"),
                        Transform::from_translation(level.center_position().extend(0.0)),
                        TilemapChunk {
                            tile_display_size: UVec2::ONE,
                            chunk_size: level.grid_size,
                            tileset: level.terrain_tileset.clone(),
                            alpha_mode: AlphaMode2d::Blend,
                        },
                        level.terrain_tiledata.clone(),
                    ));

                    // Spawn new terrain colliders
                    let terrain_colliders: Vec<_> = level
                        .terrain_colliders
                        .iter()
                        .map(|tc| {
                            let (collider, transform) = tc.into_collider_and_transform(1.0);
                            (
                                Name::new("Terrain Collider"),
                                ChildOf(level_geometry.0),
                                RigidBody::Static,
                                collider,
                                transform,
                            )
                        })
                        .collect();

                    commands.spawn_batch(terrain_colliders);
                }
                _ => {}
            }
        }
    }
}
