use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::assets::{
    level::level_collision::{LevelCollider, LevelCollisionBuilder},
    serialize::ldtk::Level as LdtkLevel,
};

mod level_collision;

#[derive(Asset, Reflect, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub grid_size: UVec2,
    pub grid_offset: IVec2,
    pub player_spawn: IVec2,
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
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let ldtk: LdtkLevel = serde_json::from_slice(&bytes)?;
        let level_offset = IVec2::new(ldtk.world_x as _, -ldtk.world_y as _);

        let entities_layer = ldtk
            .layer_instances
            .as_ref()
            .unwrap()
            .iter()
            .find(|layer| layer.identifier == "Entities")
            .unwrap();

        let player_spawn_entity = entities_layer
            .entity_instances
            .iter()
            .find(|entity| entity.identifier == "Player_Spawn")
            .unwrap();

        let player_spawn = IVec2::new(
            player_spawn_entity.grid[0] as _,
            (entities_layer.c_hei - player_spawn_entity.grid[1] - 1) as _,
        );

        let terrain_layer = ldtk
            .layer_instances
            .as_ref()
            .unwrap()
            .iter()
            .find(|layer| layer.identifier == "Terrain")
            .unwrap();

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

        Ok(Level {
            name: ldtk.identifier,
            grid_size,
            grid_offset: level_offset,
            player_spawn,
            terrain_colliders,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtkl"]
    }
}

#[cfg(feature = "dev_native")]
pub(super) mod hot_reload {
    use avian2d::prelude::{Collider, DebugRender};
    use bevy::{asset::AssetEventSystems, prelude::*};

    use crate::{assets::level::Level, demo::level::CurrentLevel};

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
        current_level: Single<(Entity, &CurrentLevel, &Children, &mut Transform)>,
        named_colliders: Query<&Name, With<Collider>>,
        mut commands: Commands,
    ) {
        let (level_id, level_handle, level_children, mut level_transform) =
            current_level.into_inner();
        for ev in asset_events.read() {
            match ev {
                &AssetEvent::Modified { id } if id == level_handle.id() => {
                    let level = levels.get(id).unwrap();
                    info!("Reloading level {:?}", level.name);

                    // Update level position
                    level_transform.translation = level.center_position().extend(0.0);

                    // Despawn existing terrain colliders
                    let despawn_batch: Vec<_> = level_children
                        .iter()
                        .filter_map(|entity| {
                            named_colliders
                                .get(entity)
                                .ok()
                                .filter(|name| name.as_str() == "Terrain Collider")
                                .map(|_| entity)
                        })
                        .collect();

                    commands.queue(move |world: &mut World| {
                        despawn_batch.into_iter().for_each(|entity| {
                            world.despawn(entity);
                        })
                    });

                    // Spawn new terrain colliders
                    let terrain_colliders: Vec<_> = level
                        .terrain_colliders
                        .iter()
                        .map(|tc| {
                            let (collider, transform) = tc.into_collider_and_transform(1.0);
                            (
                                Name::new("Terrain Collider"),
                                ChildOf(level_id),
                                collider,
                                transform,
                                DebugRender::default(),
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
