
/// The size of one texture in pixels
pub const TEXTURE_SIZE_P: usize = 8;

/// The size of texture atlas row in textures
pub const ATLAS_ROW_SIZE_T: usize = 32;

/// The size of texture atlas row in pixels
pub const ATLAS_ROW_SIZE_P: usize = TEXTURE_SIZE_P * ATLAS_ROW_SIZE_T;

/// The size of texture in unit fraction
pub const TEXTURE_SIZE_F: f32 = 1.0 / ATLAS_ROW_SIZE_T as f32;

/// Handles UV information
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
		let x_lo: f32 = (id as usize % ATLAS_ROW_SIZE_T) as f32 * TEXTURE_SIZE_F;
		let x_hi: f32 = x_lo + TEXTURE_SIZE_F;

		/* Find `Y` */
		let y_lo: f32 = (id as usize / ATLAS_ROW_SIZE_T) as f32 * TEXTURE_SIZE_F;
		let y_hi: f32 = y_lo + TEXTURE_SIZE_F;

		UV { x_lo, x_hi, y_lo, y_hi }
	}

	pub fn with_inversion(mut self) -> Self {
		self.y_lo = 1.0 - self.y_lo;
		self.y_hi = 1.0 - self.y_hi;
		return self;
	}
}