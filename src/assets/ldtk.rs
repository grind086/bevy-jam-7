use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use crate::assets::serialize::ldtk::LdtkJson;

#[derive(Asset, Reflect, Deref, Clone)]
#[reflect(opaque)]
pub struct LdtkAsset(pub LdtkJson);

#[derive(TypePath, Default)]
pub struct LdtkLoader;

impl AssetLoader for LdtkLoader {
    type Asset = LdtkAsset;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        (): &Self::Settings,
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let json = serde_json::from_slice(&bytes)?;
        Ok(LdtkAsset(json))
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}
