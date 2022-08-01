pub mod voxel_data;
pub mod atlas;
pub mod generator;

use {
	crate::app::utils::{
		cfg::{shader::voxel::light as cfg_light, self},
		math::vector::*,
		terrain::chunk::{DetailedVertex, LoweredVertex},
		terrain::voxel::VoxelData,
		reinterpreter::*,
	},
	voxel_data::*,
};

/// Represents voxel.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Generalization of voxel details.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoweredVoxel {
	pub general_color: [f32; 3],
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

	const FRONT_LIGHT:	f32 = cfg_light::FRONT;
	const BACK_LIGHT:	f32 = cfg_light::BACK;
	const TOP_LIGHT:	f32 = cfg_light::TOP;
	const BOTTOM_LIGHT:	f32 = cfg_light::BOTTOM;
	const LEFT_LIGHT:	f32 = cfg_light::LEFT;
	const RIGHT_LIGHT:	f32 = cfg_light::RIGHT;

	pub struct CubeDetailed<'c> {
		data: &'c VoxelData,
		half_size: f32,
	}

	pub struct CubeLowered {
		half_size: f32,
	}

	impl<'c> CubeDetailed<'c> {
		/// Constructs new cube maker with filled voxel data.
		pub fn new(data: &'c VoxelData) -> Self { CubeDetailed { data, half_size: cfg::terrain::VOXEL_SIZE * 0.5 } }

		/// Edit defaulted size.
		#[allow(dead_code)]
		pub fn size(mut self, new_size: f32) -> Self {
			self.half_size = new_size * 0.5;
			return self
		}

		/// Cube front face vertex array.
		pub fn front_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for front face */
			let uv = UV::new(self.data.textures.front);
			
			/* Shortcuts */
			let light = FRONT_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
		}

		/// Cube back face vertex array.
		pub fn back_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for back face */
			let uv = UV::new(self.data.textures.back);
			
			/* Shortcuts */
			let light = BACK_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
		}

		/// Cube top face vertex array.
		pub fn top_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for top face */
			let uv = UV::new(self.data.textures.top);
			
			/* Shortcuts */
			let light = TOP_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
		}

		/// Cube bottom face vertex array.
		pub fn bottom_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for bottom face */
			let uv = UV::new(self.data.textures.bottom);
			
			/* Shortcuts */
			let light = BOTTOM_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
		}

		/// Cube left face vertex array.
		pub fn left_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for left face */
			let uv = UV::new(self.data.textures.left);
			
			/* Shortcuts */
			let light = LEFT_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
		}

		/// Cube right face vertex array.
		pub fn right_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			/* UVs for right face */
			let uv = UV::new(self.data.textures.right);
			
			/* Shortcuts */
			let light = RIGHT_LIGHT;
			let (x, y, z) = (position.x() as f32, position.y() as f32, position.z() as f32);

			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_hi ], light });
			vertices.push(DetailedVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_lo, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_lo ], light });
			vertices.push(DetailedVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], tex_coords: [ uv.x_hi, uv.y_hi ], light });
		}

		/// Cube all sides.
		#[allow(dead_code)]
		pub fn all_detailed(&self, position: Int3, vertices: &mut Vec<DetailedVertex>) {
			self.left_detailed(position, vertices);
			self.right_detailed(position, vertices);
			self.front_detailed(position, vertices);
			self.back_detailed(position, vertices);
			self.top_detailed(position, vertices);
			self.bottom_detailed(position, vertices);
		}
	}

	impl CubeLowered {
		pub fn new(size: f32) -> Self {
			Self { half_size: size / 2.0 }
		}

		/// Cube front face vertex array.
		pub fn front_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = FRONT_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
		}

		/// Cube back face vertex array.
		pub fn back_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = BACK_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
		}

		/// Cube top face vertex array.
		pub fn top_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = TOP_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
		}

		/// Cube bottom face vertex array.
		pub fn bottom_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = BOTTOM_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
		}

		/// Cube left face vertex array.
		pub fn left_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = LEFT_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y, -self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y, -self.half_size + z ], color, light });
		}

		/// Cube right face vertex array.
		pub fn right_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			/* Shortcuts */
			let light = RIGHT_LIGHT;
			let (x, y, z, _) = position.as_tuple();

			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [ self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x, -self.half_size + y,  self.half_size + z ], color, light });
			vertices.push(LoweredVertex { position: [-self.half_size + x,  self.half_size + y,  self.half_size + z ], color, light });
		}

		/// Cube all sides.
		#[allow(dead_code)]
		pub fn all_lowered(&self, position: Float4, color: [f32; 3], vertices: &mut Vec<LoweredVertex>) {
			self.left_lowered(position, color, vertices);
			self.right_lowered(position, color, vertices);
			self.front_lowered(position, color, vertices);
			self.back_lowered(position, color, vertices);
			self.top_lowered(position, color, vertices);
			self.bottom_lowered(position, color, vertices);
		}}
}
