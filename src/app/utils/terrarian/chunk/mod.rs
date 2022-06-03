#![allow(dead_code)]

use super::voxel::Voxel;
use crate::app::utils::math::vector::Int3;

const CHUNK_SIZE:	usize = 16;
const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

struct Chunk {
	voxels: Box<[[[Voxel<'static>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
	pos: Int3
}

impl Chunk {
	pub fn new(pos: Int3) -> Self {
		unimplemented!();
	}
}

pub fn world_coords_to_chunk(pos: Int3) -> Int3 {
	pos / CHUNK_SIZE as i32
}

pub fn chunk_cords_to_min_world(pos: Int3) -> Int3 {
	unimplemented!();
}