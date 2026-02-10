use bevy::{
    asset::RenderAssetUsages,
    image::{ImageSampler, TextureFormatPixelInfo},
    math::USizeVec2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub struct TilesetImageBuilder {
    tile_size: USizeVec2,
    format: TextureFormat,
    px_bytes: usize,
    data: Vec<u8>,
    tiles: u16,
}

impl TilesetImageBuilder {
    pub fn new(tile_size: UVec2, format: TextureFormat) -> Option<Self> {
        Some(Self {
            tile_size: tile_size.as_usizevec2(),
            format,
            px_bytes: format.pixel_size().ok()?,
            data: Vec::new(),
            tiles: 0,
        })
    }

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

    pub fn build(self) -> Image {
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

#[derive(Debug)]
#[allow(unused)]
pub enum AddTileError {
    NoSourceData,
    IncorrectFormat {
        exp: TextureFormat,
        got: TextureFormat,
    },
    InvalidSourceOffset,
}
