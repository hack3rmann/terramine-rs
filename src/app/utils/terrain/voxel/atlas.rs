//!
//! Tools for dealing with texture atlases.
//!

use { crate::prelude::*, cfg::texture::atlas::* };

/// The size of texture atlas row in pixels
pub const ATLAS_ROW_SIZE_IN_PIXELS: usize = (ITEM_SIZE_IN_PIXELS + 2 * ITEM_PADDING_IN_PIXELS) * ITEMS_COUNT_IN_ROW;

/// The size of texture in unit fraction
pub const TEXTURE_SIZE_F: f32 = 1.0 / ITEMS_COUNT_IN_ROW as f32;

/// Padding to not hit neighbor textures
pub const ATLAS_PADDING_F: f32 = ITEM_PADDING_IN_PIXELS as f32 / ATLAS_ROW_SIZE_IN_PIXELS as f32;

/// Handles UV information.
#[derive(Clone, Copy, Debug, Default)]
pub struct UV {
    pub lo: Vec2,
    pub hi: Vec2,
}

impl UV {
    /// Gives id information to struct
    pub fn new(id: u16) -> Self {
        let mut lo = Vec2::new(
            (id as usize % ITEMS_COUNT_IN_ROW) as f32 * TEXTURE_SIZE_F,
            (id as usize / ITEMS_COUNT_IN_ROW) as f32 * TEXTURE_SIZE_F,
        );

        let mut hi = lo + Vec2::splat(TEXTURE_SIZE_F);

        /* Biasing */
        lo += Vec2::splat(BIAS);
        hi += Vec2::splat(BIAS);

        /* Applying padding */
        lo.x += ATLAS_PADDING_F;
        hi.x -= ATLAS_PADDING_F;
        lo.y += ATLAS_PADDING_F;
        hi.y -= ATLAS_PADDING_F;

        Self { lo, hi }
    }

    /// Useful if texture is inverted
    pub fn inversed(mut self) -> Self {
        self.lo.y = 1.0 - self.lo.y;
        self.hi.y = 1.0 - self.hi.y;
        self
    }
}