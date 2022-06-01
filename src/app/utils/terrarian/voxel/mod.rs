pub mod voxel_data;

use voxel_data::*;
use crate::app::utils::math::vector::*;

#[allow(dead_code)]
pub struct Voxel {
	data: &'static VoxelData,
	position: Float4
}

impl Voxel {
	
}

impl Default for Voxel {
	fn default() -> Self {
		Voxel {
			data: &FIRST_VOXEL_DATA,
			position: Default::default(),
		}
	}
}