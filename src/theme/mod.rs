//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
mod srgb_hex;
pub mod widget;

pub use srgb_hex::*;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{
        interaction::{InteractionPalette, InteractionSounds},
        palette as ui_palette, srgb_hex, widget,
    };
}

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);
}
