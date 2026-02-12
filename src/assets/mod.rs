use bevy::prelude::*;

pub mod enemy;
pub mod level;
pub mod serialize;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<level::Level>()
        .init_asset_loader::<level::LevelLoader>();

    app.init_asset::<enemy::Enemy>()
        .init_asset::<enemy::EnemyManifest>()
        .init_asset_loader::<enemy::EnemyManifestLoader>();
}
