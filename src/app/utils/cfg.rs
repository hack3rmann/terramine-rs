//! The place where all significant constants placed.

pub mod save {
	pub const META_FILE_NAME: &str = "meta.off";
	pub const STACK_FILE_EXTENTION: &str = "stk";
	pub const HEAP_FILE_EXTENTION:  &str = "hp";
}

pub mod camera {
	pub const FRUSTUM_EDGE_LINE_LENGTH: f32 = 10_000.0;
	pub const VERTICAL_LOOK_EPS: f64 = 0.001;

	pub mod default {
		pub const NEAR_PLANE:     f32 = 0.5;
		pub const FAR_PLANE:      f32 = 10_000.0;
		pub const SPEED:	      f64 = 10.0;
		pub const SPEED_FALLOFF:  f32 = 0.88;
		pub const FOV_IN_DEGREES: f32 = 60.0;
	}
}

pub mod window {
	pub mod default {
		pub const WIDTH:  f32 = 1024.0;
		pub const HEIGHT: f32 = 768.0;
	}
}

pub mod topology {
	pub const Z_FIGHTING_BIAS: f32 = 0.001;
}

pub mod terrain {
	/// Chunk side length in voxels.
	/// Must be a power of 2 due to be halfed in process of lowering details.
	pub const CHUNK_SIZE: usize = 64;

	pub const VOXEL_SIZE: f32   = 1.0;

	pub mod voxel_types {
		use crate::app::utils::terrain::voxel::voxel_data::{VoxelData, TextureSides};

		pub const VOXEL_DATA: [VoxelData; 3] = [
			VoxelData { name: "Air",					id: 0, avarage_color: [0.0, 0.0, 0.0], textures: TextureSides::all(0) },
			VoxelData { name: "Log",					id: 1, avarage_color: [1.0, 0.0, 0.0], textures: TextureSides::vertical(3, 1) },
			VoxelData { name: "Stone",					id: 2, avarage_color: [0.0, 1.0, 0.0], textures: TextureSides::all(2) },
		];
	}

	pub mod default {
		pub const WORLD_SIZES_IN_CHUNKS: [i32; 3] = [7, 1, 7];
	}
}

pub mod texture {
	pub mod atlas {
		pub const ITEM_SIZE_IN_PIXELS:    usize = 8;
		pub const ITEM_PADDING_IN_PIXELS: usize = 4;
		pub const ITEMS_COUNT_IN_ROW:     usize = 32;
		pub const BIAS:                   f32   = 0.0;
	}
}

pub mod shader {
	pub const DIRECTORY: &str = "src/shaders/";
	pub const VERTEX_FILE_EXTENTION:   &str = "vert";
	pub const FRAGMENT_FILE_EXTENTION: &str = "frag";
	pub const CLEAR_COLOR: (f32, f32, f32, f32) = (0.01, 0.01, 0.01, 1.0);
	pub const CLEAR_DEPTH: f32 = 1.0;
	pub const CLEAR_STENCIL: i32 = 0;

	pub mod voxel {
		pub mod light {
			pub const FRONT:  f32 = 0.9;
			pub const BACK:   f32 = 0.5;
			pub const TOP:    f32 = 1.0;
			pub const BOTTOM: f32 = 0.3;
			pub const LEFT:   f32 = 0.6;
			pub const RIGHT:  f32 = 0.7;
		}
	}
}