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
    pub grid_size: UVec2,
    pub grid_offset: IVec2,
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

        let terrain_layer = ldtk
            .layer_instances
            .as_ref()
            .unwrap()
            .iter()
            .find(|layer| layer.identifier == "Terrain")
            .unwrap();

        let grid_size = UVec2::new(terrain_layer.c_wid as _, terrain_layer.c_hei as _);
        let grid_offset = IVec2::new(
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
            grid_size,
            grid_offset,
            terrain_colliders,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtkl"]
    }
}
