//! Spawn the main level.

use avian2d::prelude::RigidBody;
use bevy::{
    prelude::*,
    sprite_render::{AlphaMode2d, TilemapChunk},
};

use crate::{
    asset_tracking::LoadResource,
    assets::level::Level,
    audio::music,
    demo::player::{PlayerAssets, player},
    physics::LorentzFactor,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    level: Handle<Level>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            level: assets.load("test/Level_0.ldtkl"),
        }
    }
}

#[derive(Component, Reflect, Deref)]
pub struct CurrentLevel(Handle<Level>);

#[derive(Component, Reflect)]
pub struct LevelGeometry;

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    levels: Res<Assets<Level>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let level = levels.get(&level_assets.level).unwrap();
    let level_id = commands
        .spawn((
            Name::new("Level"),
            CurrentLevel(level_assets.level.clone()),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player(
                    level.player_spawn.as_vec2(),
                    1.0,
                    &player_assets,
                    &mut texture_atlas_layouts
                ),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
            ],
        ))
        .id();

    let level_geometry_id = commands
        .spawn((
            Name::new("Level Geometry"),
            LevelGeometry,
            LorentzFactor::default(),
            Visibility::default(),
            ChildOf(level_id),
            RigidBody::Static,
            children![(
                Name::new("Terrain Tilemap"),
                Transform::from_translation(level.center_position().extend(0.0)),
                TilemapChunk {
                    tile_display_size: UVec2::ONE,
                    chunk_size: level.grid_size,
                    tileset: level.terrain_tileset.clone(),
                    alpha_mode: AlphaMode2d::Blend,
                },
                level.terrain_tiledata.clone(),
            )],
        ))
        .id();

    let terrain_colliders: Vec<_> = level
        .terrain_colliders
        .iter()
        .map(|tc| {
            let (collider, transform) = tc.into_collider_and_transform(1.0);
            (
                Name::new("Terrain Collider"),
                ChildOf(level_geometry_id),
                RigidBody::Static,
                collider,
                transform,
            )
        })
        .collect();

    commands.spawn_batch(terrain_colliders);
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
