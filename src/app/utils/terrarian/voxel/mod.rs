pub mod voxel_data;

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
	use super::*;

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
			vec! [
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: FRONT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: FRONT_LIGHT },
			]
		}

		/// Cube back face vertex array
		pub fn back(&self, position: Int3) -> Vec<Vertex> {
			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: BACK_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: BACK_LIGHT },
			]
		}

		/// Cube top face vertex array
		pub fn top(&self, position: Int3) -> Vec<Vertex> {
			vec! [
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: TOP_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: TOP_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: TOP_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: TOP_LIGHT },
			]
		}

		/// Cube bottom face vertex array
		pub fn bottom(&self, position: Int3) -> Vec<Vertex> {
			vec! [
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: BOTTOM_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: BOTTOM_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: BOTTOM_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: BOTTOM_LIGHT },
			]
		}

		/// Cube left face vertex array
		pub fn left(&self, position: Int3) -> Vec<Vertex> {
			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: LEFT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: LEFT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: LEFT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: LEFT_LIGHT },
			]
		}

		/// Cube right face vertex array
		pub fn right(&self, position: Int3) -> Vec<Vertex> {
			vec! [
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: RIGHT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ], light: RIGHT_LIGHT },
				Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ], light: RIGHT_LIGHT },
				Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ], light: RIGHT_LIGHT },
			]
		}

	}
}
