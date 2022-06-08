pub mod voxel_data;
pub mod atlas;

use voxel_data::*;
use crate::app::utils::{
	math::vector::*,
	graphics::{
		Vertex,
	},
	terrarian::voxel::VoxelData,
};

/// Represents voxel.
#[allow(dead_code)]
pub struct Voxel {
	pub data: &'static VoxelData,
	pub position: Int3,
}

impl Voxel {
	/// Voxel constructor.
	pub fn new(position: Int3, data: &'static VoxelData) -> Self {
		Voxel { data, position }
	}
}

pub mod shape {
	use super::{*, atlas::UV};

	const FRONT_LIGHT:	f32 = 0.9;
	const BACK_LIGHT:	f32 = 0.5;
	const TOP_LIGHT:	f32 = 1.0;
	const BOTTOM_LIGHT:	f32 = 0.3;
	const LEFT_LIGHT:	f32 = 0.6;
	const RIGHT_LIGHT:	f32 = 0.7;

	pub struct Cube<'c> {
		data: &'c VoxelData
	}

	impl<'c> Cube<'c> {
		/// Constructs new cube maker with filled voxel data
		pub fn new(data: &'c VoxelData) -> Self { Cube { data } }

		/// Cube front face vertex array
		pub fn front(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for front face */
			let uv = UV::new(self.data.textures.front);

			vec! [
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: FRONT_LIGHT },
			]
		}

		/// Cube back face vertex array
		pub fn back(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for back face */
			let uv = UV::new(self.data.textures.back);

			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BACK_LIGHT },
			]
		}

		/// Cube top face vertex array
		pub fn top(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for top face */
			let uv = UV::new(self.data.textures.top);

			vec! [
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: TOP_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: TOP_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: TOP_LIGHT },
			]
		}

		/// Cube bottom face vertex array
		pub fn bottom(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for bottom face */
			let uv = UV::new(self.data.textures.bottom);

			vec! [
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BOTTOM_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BOTTOM_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BOTTOM_LIGHT },
			]
		}

		/// Cube left face vertex array
		pub fn left(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for left face */
			let uv = UV::new(self.data.textures.left);

			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: LEFT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: LEFT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: LEFT_LIGHT },
			]
		}

		/// Cube right face vertex array
		pub fn right(&self, position: Int3) -> Vec<Vertex> {
			/* UVs for right face */
			let uv = UV::new(self.data.textures.right);

			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: RIGHT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: RIGHT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: RIGHT_LIGHT },
			]
		}

	}
}
