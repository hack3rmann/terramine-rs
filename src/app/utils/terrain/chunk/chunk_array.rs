use super::{MeshedChunk, MeshlessChunk, ChunkEnvironment as ChunkEnv};
use crate::app::utils::{
	graphics::Vertex,
	math::prelude::*,
	graphics::{camera::Camera, Graphics},
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
use std::sync::mpsc::{Sender, Receiver};

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

pub struct GeneratedChunkArray<'e>(MeshlessChunkArray, Vec<ChunkEnv<'e>>);

impl GeneratedChunkArray<'static> {
	pub fn generate_mesh(self) -> Receiver<(MeshlessChunkArray, Vec<Vec<Vertex>>)> {
		let (chunk_array, chunk_env) = (self.0, self.1);

		let (tx, rx) = std::sync::mpsc::channel();

		std::thread::spawn(move || {
			/* Create mesh for each chunk */
			let meshes: Vec<_> = chunk_array.chunks.iter().zip(chunk_env.iter())
				.map(|(chunk, env)| chunk.to_triangles(env))
				.collect();
			tx.send((chunk_array, meshes)).unwrap();
		});

		return rx
	}
}

unsafe impl<'e> Send for GeneratedChunkArray<'e> { }

/// Represents self-controlling chunk array.
/// * Width is bigger if you go to x+ direction
/// * Height is bigger if you go to y+ direction
/// * Depth is bigger if you go to z+ direction
#[allow(dead_code)]
pub struct MeshlessChunkArray {
	/* Size */
	width:	usize,
	height:	usize,
	depth:	usize,

	/* Chunk array itself */
	chunks: Vec<MeshlessChunk>,
}

impl MeshlessChunkArray {
	pub fn generate(width: usize, height: usize, depth: usize) -> (Receiver<GeneratedChunkArray<'static>>, Receiver<f64>) {
		/* Create channels */
		let (result_tx, result_rx) = std::sync::mpsc::channel();
		let (percenatge_tx, percentage_rx) = std::sync::mpsc::channel();

		std::thread::spawn(move || {
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
					chunks.push(MeshlessChunk::new(pos));

					/* Calculating percentage */
					let idx = (sdex::get_index(&[x, y, z], &[width, height, depth]) + 1) as f64;
					percenatge_tx.send(idx / volume as f64).unwrap();
				}}}

				/* Save */
				use SaveType::*;
				Save::new(name)
					.create(path)
					.write(&width, Width)
					.write(&height, Height)
					.write(&depth, Depth)
					.pointer_array(volume, ChunkArray, |i| {
						/* Write chunk */
						let result = if chunks[i].voxels.iter().all(|&id| id == NOTHING_VOXEL_DATA.id) {
							/* Save only chunk position if it is empty */
							let mut state = (ChunkState::Empty as u8).reinterpret_as_bytes();
							state.append(&mut chunks[i].pos.reinterpret_as_bytes());

							state
						} else {
							/* Save chunk fully */
							let mut state = (ChunkState::Full as u8).reinterpret_as_bytes();
							state.append(&mut chunks[i].reinterpret_as_bytes());

							state
						};

						/* Calculate percentage */
						let i = i + 1;
						percenatge_tx.send(i as f64 / volume as f64).unwrap();

						/* Return chunk */
						return result
					})
					.save().unwrap();
			};

			/* File reader */
			if std::path::Path::new(path).exists() {
				use SaveType::*;
				let save = Save::new(name).open(path);

				if (width, height, depth) == (save.read(Width), save.read(Height), save.read(Depth)) {
					chunks = save.read_pointer_array(ChunkArray, |mut i, bytes| {
						let elem;
						if bytes[0] == ChunkState::Full  as u8 {
							elem = MeshlessChunk::reinterpret_from_bytes(&bytes[1..])
						} else
						if bytes[0] == ChunkState::Empty as u8 {
							elem = MeshlessChunk::new(Int3::reinterpret_from_bytes(&bytes[1..]))
						}
						else {
							panic!("Unknown state ({})!", bytes[0])
						}

						/* Calculate percent */
						i += 1;
						percenatge_tx.send(i as f64 / volume as f64).unwrap();

						return elem
					});
				} else {
					generate_file()
				}
			} else {
				generate_file()
			}

			/* Make environments with references to chunk array */
			let env = Self::make_environment(&chunks, width, height, depth, None);

			/* Create and send generated data */
			let array = MeshlessChunkArray { width, height, depth, chunks };
			result_tx.send(GeneratedChunkArray(array, env)).unwrap();
		});

		/* Return reciever */
		return (result_rx, percentage_rx)
	}

	/// Creates environment for ChunkArray.
	fn make_environment<'v, 'c>(chunks: &'v Vec<MeshlessChunk>, width: usize, height: usize, depth: usize, percentage_tx: Option<Sender<f64>>) -> Vec<ChunkEnv<'c>> {
		let volume = width * height * depth;
		let mut env = vec![ChunkEnv::none(); volume];

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

			/* Calculate percentage */
			if let Some(tx) = &percentage_tx {
				let i = index(x, y, z) + 1;
				tx.send(i as f64 / volume as f64).unwrap();
			}
		}}}

		return env;
	}

	/// Gives an iterator over chunks.
	#[allow(dead_code)]
	pub fn iter(&self) -> impl Iterator<Item = &MeshlessChunk> {
		self.chunks.iter()
	}

	/// Gives an iterator over chunks.
	#[allow(dead_code)]
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut MeshlessChunk> {
		self.chunks.iter_mut()
	}

	/// Upgrades meshless chunk array to meshed.
	pub fn upgrade<'dp, 'g>(self, graphics: &'g Graphics, triangles: Vec<Vec<Vertex>>) -> MeshedChunkArray<'dp> {
		let (width, height, depth) = (self.width, self.height, self.depth);
		let chunks: Vec<_> = self.into_iter()
			.zip(triangles.into_iter())
			.map(|(chunk, triangles)| chunk.triangles_upgrade(graphics, triangles))
			.collect();

		MeshedChunkArray { width, height, depth, chunks }
	}
}

impl IntoIterator for MeshlessChunkArray {
	type Item = MeshlessChunk;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.chunks.into_iter()
	}
}

pub struct MeshedChunkArray<'dp> {
	pub width: usize,
	pub height: usize,
	pub depth: usize,

	pub chunks: Vec<MeshedChunk<'dp>>
}

impl<'dp> MeshedChunkArray<'dp> {
	/// Renders chunks.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		/* Iterating through array */
		for chunk in self.chunks.iter_mut() {
			chunk.render(target, uniforms, camera)?
		}
		Ok(())
	}
}