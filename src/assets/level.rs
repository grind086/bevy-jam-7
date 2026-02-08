use avian2d::prelude::Collider;
use bevy::{
    asset::{
        AssetLoader, AsyncWriteExt, ErasedLoadedAsset, LoadContext, LoadedAsset,
        io::{Reader, Writer},
        processor::LoadTransformAndSave,
        saver::{AssetSaver, SavedAsset},
        transformer::{AssetTransformer, TransformedAsset},
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::assets::ldtk::{LdtkAsset, LdtkLoader};

#[derive(Asset, Reflect, Serialize, Deserialize)]
pub struct Level {
    pub grid_size: UVec2,
    pub grid_offset: IVec2,
    pub terrain_colliders: Vec<TerrainCollider>,
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

#[derive(Reflect, Serialize, Deserialize, Debug, Deref, Clone, Copy)]
#[serde(transparent)]
pub struct TerrainCollider(pub URect);

impl TerrainCollider {
    pub fn into_collider(self) -> (Collider, Transform) {
        let rect = self.as_rect();
        let size = rect.size();
        let center = rect.center();
        (
            Collider::rectangle(size.x, size.y),
            Transform::from_translation(center.extend(0.0)),
        )
    }
}

pub type LevelProcess = LoadTransformAndSave<LdtkLoader, LevelTransformer, LevelSaver>;

#[derive(TypePath, Default)]
pub struct LevelTransformer;

impl AssetTransformer for LevelTransformer {
    type AssetInput = LdtkAsset;
    type AssetOutput = Level;
    type Settings = ();
    type Error = String; // You can't use `BevyError` here...

    async fn transform<'a>(
        &'a self,
        asset: TransformedAsset<Self::AssetInput>,
        &(): &'a Self::Settings,
    ) -> Result<TransformedAsset<Self::AssetOutput>, Self::Error> {
        let ldtk = asset.get();
        let level = ldtk
            .levels
            .iter()
            .find(|level| level.identifier == "Level_0")
            .unwrap();

        let terrain_layer = level
            .layer_instances
            .as_ref()
            .unwrap()
            .iter()
            .find(|layer| layer.identifier == "Terrain")
            .unwrap();

        let terrain_grid = &terrain_layer.int_grid_csv;

        let level = Level {
            grid_size: UVec2::new(terrain_layer.c_wid as _, terrain_layer.c_hei as _),
            grid_offset: IVec2::new(
                terrain_layer.px_total_offset_x as _,
                terrain_layer.px_total_offset_y as _,
            ) / terrain_layer.grid_size as i32,
            terrain_colliders: Vec::new(),
        };

        // This cannot possibly be the only way to create a `TransformedAsset` without copying
        // the source's subassets...
        Ok(TransformedAsset::from_loaded(ErasedLoadedAsset::from(
            LoadedAsset::new_with_dependencies(level),
        ))
        .expect("we had to erase the type and then downcast it"))
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

        let level = serde_json::from_slice(&bytes)?;
        Ok(level)
    }
}

#[derive(TypePath, Default)]
pub struct LevelSaver;

impl AssetSaver for LevelSaver {
    type Asset = Level;
    type Settings = ();
    type OutputLoader = LevelLoader;
    type Error = String; // You can't use `BevyError` here...

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, Self::Asset>,
        &(): &Self::Settings,
    ) -> Result<<Self::OutputLoader as AssetLoader>::Settings, Self::Error> {
        let bytes = serde_json::to_vec(asset.get()).map_err(|e| e.to_string())?;
        writer.write_all(&bytes).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
