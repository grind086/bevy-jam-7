//! Spawn the main level.

use avian2d::prelude::RigidBody;
use bevy::{
    ecs::bundle::NoBundleEffect,
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
    commands
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
        .with_children(|children| {
            let geometry_id = children
                .spawn((
                    Name::new("Level Geometry"),
                    LevelGeometry,
                    LorentzFactor::default(),
                    Visibility::default(),
                    RigidBody::Static,
                    children![tilemap(level)],
                ))
                .id();

            children
                .commands()
                .spawn_batch(colliders_batch(level, geometry_id));
        });
}

fn tilemap(level: &Level) -> impl Bundle {
    (
        Name::new("Terrain Tilemap"),
        Transform::from_translation(level.center_position().extend(0.0)),
        TilemapChunk {
            tile_display_size: UVec2::ONE,
            chunk_size: level.grid_size,
            tileset: level.terrain_tileset.clone(),
            alpha_mode: AlphaMode2d::Blend,
        },
        level.terrain_tiledata.clone(),
    )
}

fn colliders_batch(
    level: &Level,
    level_geometry: Entity,
) -> Vec<impl Bundle<Effect: NoBundleEffect>> {
    level
        .terrain_colliders
        .iter()
        .map(|tc| {
            let (collider, transform) = tc.into_collider_and_transform(1.0);
            (
                Name::new("Terrain Collider"),
                ChildOf(level_geometry),
                RigidBody::Static,
                collider,
                transform,
            )
        })
        .collect()
}

#[cfg(feature = "dev_native")]
pub(super) mod hot_reload {
    use bevy::asset::AssetEventSystems;

    use super::*;

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
                    commands.spawn((tilemap(level), ChildOf(level_geometry.0)));

                    // Spawn new terrain colliders
                    commands.spawn_batch(colliders_batch(level, level_geometry.0));
                }
                _ => {}
            }
        }
    }
}
