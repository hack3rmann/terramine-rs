use super::{Chunk, ChunkEnvironment as ChunkEnv};
use crate::app::utils::{
	graphics::Graphics,
	math::prelude::*,
	graphics::camera::Camera,
	saves::*,
	reinterpreter::{
		ReinterpretAsBytes,
		ReinterpretFromBytes
	},
	terrain::voxel::voxel_data::NOTHING_VOXEL_DATA,
};
use glium::{
	uniforms::Uniforms,
	DrawError,
	Frame
};

/// Represents self-controlling chunk array.
/// * Width is bigger if you go to x+ direction
/// * Height is bigger if you go to y+ direction
/// * Depth is bigger if you go to z+ direction
#[allow(dead_code)]
pub struct ChunkArray<'a> {
	/* Size */
	width:	usize,
	height:	usize,
	depth:	usize,

	/* Chunk array itself */
	chunks: Vec<Chunk<'a>>,
}

#[derive(Clone, Copy)]
enum SaveType {
	Width,
	Height,
	Depth,
	ChunkArray,
}

impl Into<Offset> for SaveType {
	fn into(self) -> Offset { self as Offset }
}

enum ChunkState {
	Full,
	Empty,
}

impl<'a> ChunkArray<'a> {
	pub fn new(graphics: &Graphics, width: usize, height: usize, depth: usize) -> Self {
		/* Amount of voxels in chunks */
		let volume = width * height * depth;

		/* Initialize vector */
		let mut chunks = vec![];

		/* Name of world file */
		let (path, name) = ("src/world", "world");

		/* File generator */
		let mut generate_file = || {
			/* Generate chunks */
			chunks = Vec::with_capacity(volume);
			for x in 0..width {
			for y in 0..height {
			for z in 0..depth {
				let pos = Int3::new(x as i32, y as i32, z as i32) - Int3::new(width as i32, height as i32, depth as i32) / 2;
				chunks.push(Chunk::new(None, pos, false));
			}}}

			/* Save */
			use SaveType::*;
			Save::new(name)
				.create(path)
				.write(&width, Width)
				.write(&height, Height)
				.write(&depth, Depth)
				.pointer_array(volume, ChunkArray, |i|
					if chunks[i].voxels.iter().all(|&id| id == NOTHING_VOXEL_DATA.id) {
						/* Save only chunk position if it is empty */
						let mut state = (ChunkState::Empty as u8).reinterpret_as_bytes();
						state.append(&mut chunks[i].pos.reinterpret_as_bytes());

						state
					} else {
						/* Save chunk fully */
						let mut state = (ChunkState::Full as u8).reinterpret_as_bytes();
						state.append(&mut chunks[i].reinterpret_as_bytes());

						state
					}
				)
				.save().unwrap();
		};

		/* File reader */
		if std::path::Path::new(path).exists() {
			use SaveType::*;
			let save = Save::new(name).open(path);

			if (width, height, depth) == (save.read(Width), save.read(Height), save.read(Depth)) {
				chunks = save.read_pointer_array(ChunkArray, |bytes|
					if bytes[0] == ChunkState::Full as u8 {
						Chunk::reinterpret_from_bytes(&bytes[1..])
					} else {
						Chunk::new(None, Int3::reinterpret_from_bytes(&bytes[1..]), false)
					}
				);
			} else {
				generate_file()
			}
		} else {
			generate_file()
		}

		/* Make environments with references to chunk array */
		let env = Self::make_environment(&chunks, width, height, depth);

		/* Create mesh for each chunk */
		chunks.iter().zip(env.iter())
			.for_each(|(chunk, env)| chunk.update_mesh(&graphics, env));

		ChunkArray { width, height, depth, chunks }
	}

	/// Creates environment for ChunkArray
	fn make_environment<'v, 'c>(chunks: &'v Vec<Chunk<'c>>, width: usize, height: usize, depth: usize) -> Vec<ChunkEnv<'c>> {
		let mut env = vec![ChunkEnv::none(); width * height * depth];

		for x in 0..width {
		for y in 0..height {
		for z in 0..depth {
			/* Local index function */
			let index = |x, y, z| sdex::get_index(&[x, y, z], &[width, height, depth]);

			/* Reference to current environment variable */
			let env = &mut env[index(x, y, z)];

			/* For `front` side */
			if x as isize - 1 >= 0 {
				env.front	= Some(&chunks[index(x - 1, y, z)]);
			}

			/* For `back` side */
			if x + 1 < width {
				env.back	= Some(&chunks[index(x + 1, y, z)]);
			}

			/* For `bottom` side */
			if y as isize - 1 >= 0 {
				env.bottom	= Some(&chunks[index(x, y - 1, z)]);
			}
		
			/* For `top` side */
			if y + 1 < height {
				env.top		= Some(&chunks[index(x, y + 1, z)]);
			}

			/* For `left` side */
			if z as isize - 1 >= 0 {
				env.left	= Some(&chunks[index(x, y, z - 1)]);
			}

			/* For `right` side */
			if z + 1 < depth {
				env.right	= Some(&chunks[index(x, y, z + 1)]);
			}
		}}}

		return env;
	}

	/// Renders chunks.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		/* Iterating through array */
		for chunk in self.chunks.iter_mut() {
			chunk.render(target, uniforms, camera)?
		}
		Ok(())
	}
}