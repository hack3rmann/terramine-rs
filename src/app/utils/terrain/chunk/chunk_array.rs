use {
	crate::app::utils::{
		werror::prelude::*,
		math::prelude::*,
		graphics::{
			camera::Camera,
			Graphics,
			shader::Shader,
			debug_visuals::*,
		},
		saves::*,
		reinterpreter::*,
		concurrency::{
			loading::Loading,
			promise::Promise,
		},
	},
	super::{
		MeshedChunk,
		MeshlessChunk,
		ChunkEnvironment as ChunkEnv,
		ChunkFill,
		Addition,
		ChunkDetails,
		Detailed,
		DetailedVertexVec,
	},
	glium::{
		uniforms::Uniforms,
		DrawError,
		Frame,
		DrawParameters,
		Depth,
		DepthTest,
		BackfaceCullingMode,
	},
	std::sync::mpsc::Sender,
};

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

pub struct GeneratedChunkArray<'e>(MeshlessChunkArray, Vec<ChunkEnv<'e>>);

impl GeneratedChunkArray<'static> {
	pub fn generate_mesh(self, percentage_tx: Sender<Loading>) -> (MeshlessChunkArray, Vec<DetailedVertexVec>) {
		let GeneratedChunkArray(chunk_array, chunk_env) = self;
		let volume = chunk_array.width * chunk_array.height * chunk_array.depth;

		/* Create mesh for each chunk */
		let meshes: Vec<_> = chunk_array.chunks.iter()
			.zip(chunk_env.iter())
			.zip(1_usize..)
			.map(|((chunk, env), i)| {
				/* Get mesh */
				let result = chunk.to_triangles(env);

				/* Calculate percentage */
				percentage_tx.send(Loading::from_range("Mesh generation", i, 0..volume)).wunwrap();

				return result
			})
			.collect();

		(chunk_array, meshes)
	}
}

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
	pub fn generate(width: usize, height: usize, depth: usize) -> (Promise<(MeshlessChunkArray, Vec<DetailedVertexVec>)>, Promise<Loading>) {
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
					let idx = sdex::get_index(&[x, y, z], &[width, height, depth]);
					percenatge_tx.send(Loading::from_range("Chunk generation", idx, 0..volume)).wunwrap();
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
						let result = if chunks[i].is_empty() {
							/* Save only chunk position if it is empty */
							let mut state = ChunkFill::Empty.reinterpret_as_bytes();
							state.append(&mut chunks[i].pos.reinterpret_as_bytes());

							state
						} else if chunks[i].is_filled() {
							/* Save only chunk position and one id */
							let id = chunks[i].fill_id().wunwrap();
							let mut state = ChunkFill::All(id).reinterpret_as_bytes();
							state.append(&mut chunks[i].pos.reinterpret_as_bytes());

							state
						} else {
							/* Save chunk fully */
							let mut state = ChunkFill::Standart.reinterpret_as_bytes();
							state.append(&mut chunks[i].reinterpret_as_bytes());

							state
						};

						/* Calculate percentage */
						percenatge_tx.send(Loading::from_range("Saving to file", i, 0..volume)).wunwrap();

						/* Return chunk */
						return result
					})
					.save().wunwrap();
			};

			/* File reader */
			if std::path::Path::new(path).exists() {
				use SaveType::*;
				let save = Save::new(name).open(path);

				if (width, height, depth) == (save.read(Width), save.read(Height), save.read(Depth)) {
					chunks = save.read_pointer_array(ChunkArray, |i, bytes| {
						let offset = ChunkFill::static_size();
						let chunk_fill = ChunkFill::reinterpret_from_bytes(&bytes[0..offset]);

						/* Read chunk from bytes */
						let result = match chunk_fill {
							ChunkFill::Empty => {
								let pos = Int3::reinterpret_from_bytes(&bytes[offset..]);
								MeshlessChunk::new_empty(pos)
							},
							ChunkFill::All(id) => {
								let pos = Int3::reinterpret_from_bytes(&bytes[offset..]);
								MeshlessChunk::new_filled(pos, id)
							},
							ChunkFill::Standart => {
								let mut chunk = MeshlessChunk::reinterpret_from_bytes(&bytes[offset..]);
								chunk.additional_data = Addition::Know {
									fill: Some(ChunkFill::Standart),
									details: ChunkDetails::Full
								};
								chunk
							},
						};

						/* Calculate percent */
						percenatge_tx.send(Loading::from_range("Reading from file", i, 0..volume)).wunwrap();

						return result
					});
				} else {
					generate_file()
				}
			} else {
				generate_file()
			}

			/* Make environments with references to chunk array */
			let env = Self::make_environment(&chunks, width, height, depth, Some(percenatge_tx.clone()));

			/* Create generated data */
			let array = MeshlessChunkArray { width, height, depth, chunks };
			let result = GeneratedChunkArray(array, env).generate_mesh(percenatge_tx);

			/* Send */
			result_tx.send(result).wunwrap();
		});

		/* Return reciever */
		return (Promise(result_rx), Promise(percentage_rx))
	}

	/// Creates environment for ChunkArray.
	fn make_environment<'v, 'c>(chunks: &'v Vec<MeshlessChunk>, width: usize, height: usize, depth: usize, percentage_tx: Option<Sender<Loading>>) -> Vec<ChunkEnv<'c>> {
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
				let i = index(x, y, z);
				tx.send(Loading::from_range("Calculating environment", i, 0..volume)).wunwrap();
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
	pub fn upgrade<'g, 'dp>(self, graphics: &'g Graphics, triangles: Vec<DetailedVertexVec>) -> MeshedChunkArray<'dp> {
		let (width, height, depth) = (self.width, self.height, self.depth);
		let chunks: Vec<_> = self.into_iter()
			.zip(triangles.into_iter())
			.map(|(chunk, triangles)| {
				let triangles = match &triangles {
					Detailed::Full(vec) => Detailed::Full(&vec[..]),
					Detailed::Low(vec) => Detailed::Low(&vec[..]),
				};
				let chunk = chunk.triangles_upgrade(graphics, triangles);
				DebugVisualized::new_meshed_chunk(chunk, &graphics.display)
			})
			.collect();

		/* Chunk draw parameters */
		let draw_params = DrawParameters {
			depth: Depth {
				test: DepthTest::IfLess,
				write: true,
				.. Default::default()
			},
			backface_culling: BackfaceCullingMode::CullClockwise,
			.. Default::default()
		};
		
		/* Create shader */
		let shader = Shader::new("vertex_shader", "fragment_shader", &graphics.display);

		MeshedChunkArray { width, height, depth, chunks, shader, draw_params }
	}
}

impl IntoIterator for MeshlessChunkArray {
	type Item = MeshlessChunk;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.chunks.into_iter()
	}
}

pub struct MeshedChunkArray<'a> {
	pub width: usize,
	pub height: usize,
	pub depth: usize,

	pub chunks: Vec<DebugVisualized<MeshedChunk>>,
	pub shader: Shader,
	pub draw_params: DrawParameters<'a>
}

impl<'a> MeshedChunkArray<'a> {
	/// Renders chunks.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		/* Iterating through array */
		for chunk in self.chunks.iter_mut() {
			chunk.render_meshed_chunks(target, &self.shader, uniforms, &self.draw_params, camera)?;
		}
		Ok(())
	}
}