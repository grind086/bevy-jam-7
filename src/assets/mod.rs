use bevy::prelude::*;

pub mod level;
pub mod serialize;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<level::Level>()
        .init_asset_loader::<level::LevelLoader>();
}
