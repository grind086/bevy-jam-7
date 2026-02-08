//! Spawn the main level.

use avian2d::prelude::DebugRender;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    assets::level::Level,
    audio::music,
    demo::player::{PlayerAssets, player},
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
            Transform::from_translation(level.center_position().extend(0.0)),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player(45.0, &player_assets, &mut texture_atlas_layouts),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                )
            ],
        ))
        .id();

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
