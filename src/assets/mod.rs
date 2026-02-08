use bevy::prelude::*;

pub mod ldtk;
pub mod level;
pub mod serialize;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<ldtk::LdtkAsset>()
        .init_asset_loader::<ldtk::LdtkLoader>()
        .init_asset::<level::Level>()
        .init_asset_loader::<level::LevelLoader>()
        .register_asset_processor(level::LevelProcess::new(default(), default()));
}
