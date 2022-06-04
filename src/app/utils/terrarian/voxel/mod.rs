pub mod voxel_data;

use voxel_data::*;
use crate::app::utils::{
	math::vector::*,
	graphics::{
		Vertex,
	},
};

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

	/// Gives vertex vector from position.
	pub fn cube(position: Int3) -> Vec<Vertex> {
		vec! [
			/* Front */
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			/* Back */
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			/* Left */
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			/* Right */
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			/* Top */
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			/* Bottom */
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
		]
	}

	pub fn cube_front(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
		]
	}

	pub fn cube_back(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
		]
	}

	pub fn cube_top(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
		]
	}

	pub fn cube_bottom(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
		]
	}

	pub fn cube_left(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
		]
	}

	pub fn cube_right(position: Int3) -> Vec<Vertex> {
		vec! [
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ 1.0, 1.0 ] },
		]
	}
}
