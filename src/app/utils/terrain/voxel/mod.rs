pub mod voxel_data;
pub mod atlas;
pub mod generator;

use {
	crate::app::utils::{
		math::vector::*,
		graphics::Vertex,
		terrain::voxel::VoxelData,
		reinterpreter::*,
	},
	voxel_data::*,
};

/// Represents voxel.
#[derive(Debug, PartialEq)]
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



unsafe impl Reinterpret for Voxel { }

unsafe impl ReinterpretAsBytes for Voxel {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(Self::static_size());

		bytes.append(&mut self.data.id.reinterpret_as_bytes());
		bytes.append(&mut self.position.reinterpret_as_bytes());

		return bytes;
	}
}

unsafe impl ReinterpretFromBytes for Voxel {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let id = u32::reinterpret_from_bytes(&source[0..4]);
		let pos = Int3::reinterpret_from_bytes(&source[4..16]);

		Self::new(pos, &VOXEL_DATA[id as usize])
	}
}

unsafe impl ReinterpretSize for Voxel {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for Voxel {
	fn static_size() -> usize { 16 }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reinterpret_voxel1() {
		let before = Voxel::new(Int3::new(123, 4212, 11), STONE_VOXEL_DATA);
		let after = Voxel::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_voxel2() {
		let before = Voxel::new(Int3::new(-213, 4212, 11), LOG_VOXEL_DATA);
		let after = Voxel::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
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
		pub fn front(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for front face */
			let uv = UV::new(self.data.textures.front);

			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: FRONT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: FRONT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: FRONT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: FRONT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: FRONT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: FRONT_LIGHT });
		}

		/// Cube back face vertex array
		pub fn back(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for back face */
			let uv = UV::new(self.data.textures.back);

			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BACK_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BACK_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BACK_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BACK_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BACK_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BACK_LIGHT });
		}

		/// Cube top face vertex array
		pub fn top(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for top face */
			let uv = UV::new(self.data.textures.top);

			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: TOP_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: TOP_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: TOP_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: TOP_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: TOP_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: TOP_LIGHT });
		}

		/// Cube bottom face vertex array
		pub fn bottom(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for bottom face */
			let uv = UV::new(self.data.textures.bottom);

			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BOTTOM_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BOTTOM_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: BOTTOM_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: BOTTOM_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: BOTTOM_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: BOTTOM_LIGHT });
		}

		/// Cube left face vertex array
		pub fn left(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for left face */
			let uv = UV::new(self.data.textures.left);

			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: LEFT_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: LEFT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: LEFT_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: LEFT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: LEFT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32, -0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: LEFT_LIGHT });
		}

		/// Cube right face vertex array
		pub fn right(&self, position: Int3, vertices: &mut Vec<Vertex>) {
			/* UVs for right face */
			let uv = UV::new(self.data.textures.right);

			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: RIGHT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: RIGHT_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_hi ], light: RIGHT_LIGHT });
			vertices.push(Vertex { position: [ 0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_lo, uv.y_lo ], light: RIGHT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32, -0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_lo ], light: RIGHT_LIGHT });
			vertices.push(Vertex { position: [-0.5 + position.x() as f32,  0.5 + position.y() as f32,  0.5 + position.z() as f32 ], tex_coords: [ uv.x_hi, uv.y_hi ], light: RIGHT_LIGHT });
		}

	}
}
