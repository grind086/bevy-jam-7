use bevy::prelude::*;

pub mod level;
pub mod serialize;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<level::Level>()
        .init_asset_loader::<level::LevelLoader>();

    #[cfg(feature = "dev_native")]
    {
        app.add_plugins(level::hot_reload::plugin);
    }
}
