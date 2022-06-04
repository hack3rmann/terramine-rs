#![allow(dead_code)]

use super::voxel::Voxel;
use super::voxel::voxel_data::GRASS_VOXEL_DATA;
use crate::app::utils::{
	math::vector::{
		Int3,
		swizzle::*,
	},
	graphics::Graphics
};
use glium::{
	DrawError,
	uniforms::Uniforms,
	Frame
};

/// Predefined chunk values.
const CHUNK_SIZE:	usize = 8;
const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Voxel<'static>>;

/// Chunk struct.
pub struct Chunk {
	voxels: VoxelArray,
	pos: Int3
}

impl Chunk {
	/// Constructs new chunk in given position 
	pub fn new(graphics: &Graphics, pos: Int3) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(CHUNK_VOLUME);

		/* Iterating in the chunk */
		for x in 0..=CHUNK_SIZE {
		for y in 0..=CHUNK_SIZE {
		for z in 0..=CHUNK_SIZE {
			voxels.push(Voxel::new(graphics, pos_in_chunk_to_world(Int3::new(x as i32, y as i32, z as i32), pos), &GRASS_VOXEL_DATA));
		}}}
		
		Chunk { voxels, pos }
	}

	/// Renders chunk.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U) -> Result<(), DrawError> {
		/* Iterating through array */
		for voxel in self.voxels.iter() {
			voxel.mesh.render(target, uniforms)?
		}

		Ok(( ))
	}
}

/// Transforms world coordinates to chunk 
pub fn world_coords_to_chunk(pos: Int3) -> Int3 {
	pos / CHUNK_SIZE as i32
}

/// Transforms chunk coords to world
pub fn chunk_cords_to_min_world(pos: Int3) -> Int3 {
	pos * CHUNK_SIZE as i32
}

/// Transforms in-chunk coords to world
pub fn pos_in_chunk_to_world(in_chunk: Int3, chunk: Int3) -> Int3 {
	chunk_cords_to_min_world(chunk) + in_chunk
}