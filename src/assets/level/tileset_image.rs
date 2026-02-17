use bevy::{
    asset::RenderAssetUsages,
    image::{ImageSampler, TextureFormatPixelInfo},
    math::USizeVec2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use thiserror::Error;

/// Used to build a 2d layered tileset [`Image`] from one or more source images.
pub struct TilesetImageBuilder {
    tile_size: USizeVec2,
    format: TextureFormat,
    px_bytes: usize,
    data: Vec<u8>,
    tiles: u16,
}

impl TilesetImageBuilder {
    /// Create a new tileset image builder using the given tile size and [`TextureFormat`].
    pub fn new(tile_size: UVec2, format: TextureFormat) -> Result<Self, UnsupportedFormatError> {
        Ok(Self {
            tile_size: tile_size.as_usizevec2(),
            format,
            px_bytes: format
                .pixel_size()
                .map_err(|_| UnsupportedFormatError(format))?,
            data: Vec::new(),
            tiles: 0,
        })
    }

    /// Copies the tile from the source image at the given pixel offset, and returns its id in
    /// the tileset being built.
    pub fn add_tile(
        &mut self,
        source_image: &Image,
        source_offset: UVec2,
    ) -> Result<u16, AddTileError> {
        if source_image.texture_descriptor.format != self.format {
            return Err(AddTileError::IncorrectFormat {
                exp: self.format,
                got: source_image.texture_descriptor.format,
            });
        }

        let source_data = source_image
            .data
            .as_ref()
            .ok_or(AddTileError::NoSourceData)?;

        let linear_offset = source_offset.x + source_image.width() * source_offset.y;

        let byte_offset = linear_offset as usize * self.px_bytes;
        let srow_bytes = source_image.width() as usize * self.px_bytes;
        let trow_bytes = self.tile_size.x * self.px_bytes;

        let last_byte = byte_offset + (self.tile_size.y - 1) * srow_bytes + trow_bytes;
        if last_byte > source_data.len() {
            return Err(AddTileError::InvalidSourceOffset);
        }

        for r in 0..self.tile_size.y {
            let i = byte_offset + r * srow_bytes;
            let j = i + trow_bytes;
            self.data.extend_from_slice(&source_data[i..j]);
        }

        Ok(self.next_tile_id())
    }

    /// Returns the final tileset [`Image`].
    pub fn build(mut self) -> Image {
        info!("Built tileset with {} tiles", self.tiles);

        // Fix an error where wgpu-hal heuristically decides that D2 array textures with mod 6
        // layers should be cubemaps.
        if self.tiles % 6 == 0 {
            info!("Inserting dummy tile to fix wgpu-hal issue");
            let tile_bytes = self.tile_size.element_product() * self.px_bytes;
            self.data.extend(core::iter::repeat(0).take(tile_bytes));
            self.tiles += 1;
        }

        let mut image = Image::new(
            Extent3d {
                width: self.tile_size.x as _,
                height: self.tile_size.y as _,
                depth_or_array_layers: self.tiles as _,
            },
            TextureDimension::D2,
            self.data,
            self.format,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::nearest();
        image
    }

    fn next_tile_id(&mut self) -> u16 {
        let n = self.tiles;
        self.tiles += 1;
        n
    }
}

/// Returned when attempting to construct a [`TilesetImageBuilder`] with an unsupported
/// [`TextureFormat`].
#[derive(Debug, Error)]
#[error("source image format {0:?} is unsupported")]
pub struct UnsupportedFormatError(pub TextureFormat);

/// Errors returned by [`TilesetImageBuilder::add_tile`].
#[derive(Debug, Error)]
pub enum AddTileError {
    /// The source's [`Image::data`] was `None`.
    #[error("source image is uninitialized")]
    NoSourceData,
    /// The source image was in a different format than the builder is using.
    #[error("expected source image to be in format {exp:?}, but it was {got:?}")]
    IncorrectFormat {
        exp: TextureFormat,
        got: TextureFormat,
    },
    /// The pixel offset into the source image was invalid.
    #[error("the source tile extends beyond the source image's bounds")]
    InvalidSourceOffset,
}
