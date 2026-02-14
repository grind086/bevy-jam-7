//! Spawn the main level.

use avian2d::prelude::{CollisionLayers, LinearVelocity, RigidBody};
use bevy::{
    ecs::bundle::NoBundleEffect,
    prelude::*,
    sprite_render::{AlphaMode2d, TilemapChunk},
};
use rand::Rng;

use crate::{
    PausableSystems,
    animation::AnimationPlayer,
    asset_tracking::LoadResource,
    assets::{
        enemy::{Enemy, EnemyManifest},
        level::Level,
    },
    audio::music,
    demo::{
        movement::{GroundNormal, MovementIntent, movement_controller},
        player::{PlayerAssets, player},
    },
    physics::{GamePhysicsLayersExt, LorentzFactor},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>().add_systems(
        Update,
        (update_enemy_intents, update_enemy_animations)
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    #[cfg(feature = "dev_native")]
    {
        app.add_plugins(hot_reload::plugin);
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    level: Handle<Level>,
    #[dependency]
    enemies: Handle<EnemyManifest>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            level: assets.load("test/Level_0.ldtkl"),
            enemies: assets.load("enemies.json"),
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
    enemy_manifest: Res<Assets<EnemyManifest>>,
    enemies: Res<Assets<Enemy>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let level = levels.get(&level_assets.level).unwrap();
    let enemy_manifest = enemy_manifest.get(&level_assets.enemies).unwrap();
    commands
        .spawn((
            Name::new("Level"),
            CurrentLevel(level_assets.level.clone()),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player(
                    level.player_spawn,
                    &player_assets,
                    &mut texture_atlas_layouts
                ),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone(), 1.0)
                ),
                (
                    Name::new("Enemies"),
                    Transform::default(),
                    Visibility::default(),
                    Children::spawn(SpawnIter(
                        enemies_vec(enemy_manifest, &enemies, level).into_iter()
                    ))
                )
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
                CollisionLayers::level_geometry(),
                collider,
                transform,
            )
        })
        .collect()
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyHandle(Handle<Enemy>);

fn enemies_vec(
    enemy_manifest: &EnemyManifest,
    enemies: &Assets<Enemy>,
    level: &Level,
) -> Vec<impl Bundle> {
    level
        .enemy_spawns
        .iter()
        .filter_map(|spawn| {
            let Some(handle) = enemy_manifest.enemies.get(&spawn.label) else {
                warn!("Unknown enemy label: {:?}", spawn.label);
                return None;
            };

            let enemy = enemies.get(handle)?;
            Some((
                Name::new(format!("Enemy: {}", enemy.name)),
                EnemyHandle(handle.clone()),
                Sprite {
                    image: enemy.atlas.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: enemy.atlas_layout.clone(),
                        index: 0,
                    }),
                    custom_size: Some(enemy.size),
                    ..default()
                },
                AnimationPlayer::from(enemy.idle_anim.clone()),
                Transform::from_translation((spawn.position - enemy.collider_offset).extend(0.0)),
                movement_controller(
                    enemy.movement.clone(),
                    enemy.collider.clone(),
                    enemy.collider_offset,
                    CollisionLayers::enemy(),
                ),
                MovementIntent {
                    direction: 1.0,
                    jump: true,
                },
            ))
        })
        .collect::<Vec<_>>()
}

fn update_enemy_intents(mut query: Query<&mut MovementIntent, With<EnemyHandle>>) {
    for mut intent in &mut query {
        if rand::rng().random_bool(0.01) {
            intent.direction = if rand::rng().random_bool(0.5) {
                1.0
            } else {
                -1.0
            };
        }
        intent.jump = rand::rng().random_bool(0.01);
    }
}

fn update_enemy_animations(
    assets: Res<Assets<Enemy>>,
    mut player_query: Query<(
        &EnemyHandle,
        &MovementIntent,
        Option<&GroundNormal>,
        Option<&LinearVelocity>,
        &mut Sprite,
        &mut AnimationPlayer,
    )>,
) {
    for (handle, intent, ground_norm, velocity, mut sprite, mut animation) in &mut player_query {
        let Some(enemy) = assets.get(&handle.0) else {
            continue;
        };

        if intent.direction != 0.0 {
            sprite.flip_x = intent.direction < 0.0;
        }

        let next_anim = if ground_norm.is_none_or(GroundNormal::is_grounded) {
            if intent.direction == 0.0 {
                &enemy.idle_anim
            } else {
                &enemy.walk_anim
            }
        } else {
            let v = velocity.map_or(-1.0, |v| v.y);
            if v.abs() < 0.5 {
                &enemy.peak_anim
            } else if v > 0.0 {
                &enemy.jump_anim
            } else {
                &enemy.fall_anim
            }
        };

        if next_anim.id() != animation.animation.id() {
            animation.animation = next_anim.clone();
        }
    }
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
