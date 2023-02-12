/*
 * Control section 
 */

use crate::app::utils::cfg::texture::atlas as cfg_atlas;

/// The size of one texture in pixels
pub const TEXTURE_SIZE_P: usize = cfg_atlas::ITEM_SIZE_IN_PIXELS;

/// Padding to not hit neighbor textures
pub const ATLAS_PADDING_P: usize = cfg_atlas::ITEM_PADDING_IN_PIXELS;

/// The size of texture atlas row in textures
pub const ATLAS_ROW_SIZE_T: usize = cfg_atlas::ITEMS_COUNT_IN_ROW;

/// Bias to not hit neighbor textures
pub const ATLAS_BIAS: f32 = cfg_atlas::BIAS;

/*
 * Auto constants section
 */

/// The size of texture atlas row in pixels
pub const ATLAS_ROW_SIZE_P: usize = (TEXTURE_SIZE_P + 2 * ATLAS_PADDING_P) * ATLAS_ROW_SIZE_T;

/// The size of texture in unit fraction
pub const TEXTURE_SIZE_F: f32 = 1.0 / ATLAS_ROW_SIZE_T as f32;

/// Padding to not hit neighbor textures
pub const ATLAS_PADDING_F: f32 = ATLAS_PADDING_P as f32 / ATLAS_ROW_SIZE_P as f32;

/// Handles UV information
#[derive(Clone, Copy)]
pub struct UV {
    pub x_lo: f32,
    pub x_hi: f32,
    pub y_lo: f32,
    pub y_hi: f32,
}

impl UV {
    /// Gives id information to struct
    pub fn new(id: u16) -> Self {
        /* Find `X` */
        let mut x_lo: f32 = (id as usize % ATLAS_ROW_SIZE_T) as f32 * TEXTURE_SIZE_F;
        let mut x_hi: f32 = x_lo + TEXTURE_SIZE_F;

        /* Find `Y` */
        let mut y_lo: f32 = (id as usize / ATLAS_ROW_SIZE_T) as f32 * TEXTURE_SIZE_F;
        let mut y_hi: f32 = y_lo + TEXTURE_SIZE_F;

        /* Biasing */
        x_lo += ATLAS_BIAS;
        x_hi += ATLAS_BIAS;
        y_lo += ATLAS_BIAS;
        y_hi += ATLAS_BIAS;

        /* Applying padding */
        x_lo += ATLAS_PADDING_F;
        x_hi -= ATLAS_PADDING_F;
        y_lo += ATLAS_PADDING_F;
        y_hi -= ATLAS_PADDING_F;

        UV { x_lo, x_hi, y_lo, y_hi }.with_inversion()
    }

    /// Useful if texture is inverted
    pub fn with_inversion(mut self) -> Self {
        self.y_lo = 1.0 - self.y_lo;
        self.y_hi = 1.0 - self.y_hi;
        return self;
    }
}