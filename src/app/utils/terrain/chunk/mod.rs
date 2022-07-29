pub mod chunk_array;

use {
	crate::app::utils::{
		werror::prelude::*,
		math::prelude::*,
		graphics::{
			Graphics,
			mesh::{Mesh, UnindexedMesh},
			Vertex,
			shader::Shader,
			vertex_buffer::VertexBuffer,
			camera::Camera,
		},
		reinterpreter::*,
	},
	super::voxel::{
		Voxel,
		shape::Cube,
		voxel_data::*,
		generator,
	},
	glium::{
		DrawError,
		uniforms::Uniforms,
		Frame,
		index::PrimitiveType
	},
	std::{cell::RefCell, marker::PhantomData, borrow::Cow},
};

/**
 * Index cheatsheet!
 * 
 * i = d(hx + y) + z
 * 
 * x = (i / d) / h
 * y = (i / d) % h
 * z = i % d
 */

/// Predefined chunk values.
pub const CHUNK_SIZE:	usize = 64;
pub const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Id>;

pub enum FindOptions {
	Border,
	InChunkNothing,
	InChunkSome(Voxel)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkFill {
	Empty,
	All(Id),

	#[default]
	Other,
}

unsafe impl Reinterpret for ChunkFill { }

unsafe impl ReinterpretAsBytes for ChunkFill {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		match self {
			Self::Empty => vec![0; Self::static_size()],
			Self::Other => {
				let mut result = vec![0; Self::static_size()];
				result[0] = 2;

				return result
			},
			Self::All(id) => {
				let mut result = vec![1];
				result.append(&mut id.reinterpret_as_bytes());

				return result
			},
		}
	}
}

unsafe impl ReinterpretFromBytes for ChunkFill {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		match source[0] {
			0 => Self::Empty,
			1 => {
				let id = Id::reinterpret_from_bytes(&source[1..]);
				return Self::All(id)
			},
			2 => Self::Other,
			_ => unreachable!("There's no ChunkFill variant that matches with {}!", source[0])
		}
	}
}

unsafe impl ReinterpretSize for ChunkFill {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for ChunkFill {
	fn static_size() -> usize { u8::static_size() + Id::static_size() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AdditionalData {
	pub fill: ChunkFill,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Addition {
	#[default]
	NothingToKnow,
	Know(AdditionalData),
}

impl AsRef<Self> for Addition {
	fn as_ref(&self) -> &Self { self }
}

/// Chunk struct.
pub struct MeshlessChunk {
	pub voxels: VoxelArray,
	pub pos: Int3,
	pub additional_data: Addition,
}

impl MeshlessChunk {
	/// Constructs new chunk in given position 
	pub fn new(pos: Int3) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(CHUNK_VOLUME);

		/* Additional data */
		let mut data = AdditionalData { fill: ChunkFill::Empty };

		/* Iterating in the chunk */
		for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			let global_pos = pos_in_chunk_to_world(Int3::new(x as i32, y as i32, z as i32), pos);

			/* Update addidional chunk data */
			let mut update_data = |id| {
				match data.fill {
					ChunkFill::Empty => {
						/* Check for first iteration */
						data.fill = if (x, y, z) != (0, 0, 0) {
							ChunkFill::Other
						} else {
							ChunkFill::All(id)
						}
					},
					ChunkFill::All(all_id) => {
						if all_id != id {
							data.fill = ChunkFill::Other;
						}
					}
					ChunkFill::Other => (),
				}
			};

			/* Kind of trees generation */
			if generator::trees(global_pos) {
				let id = LOG_VOXEL_DATA.id;
				update_data(id);
				voxels.push(id);
			}
			
			/* Sine-like floor */
			else if generator::sine(global_pos) {
				let id = STONE_VOXEL_DATA.id;
				update_data(id);
				voxels.push(id);
			}
			
			/* Air */
			else {
				if let ChunkFill::All(_) = data.fill {
					data.fill = ChunkFill::Other
				}

				voxels.push(NOTHING_VOXEL_DATA.id)
			}
		}}}

		/* Chunk is empty so array can be empty */
		if let ChunkFill::Empty   = data.fill { voxels = vec![  ] }
		if let ChunkFill::All(id) = data.fill { voxels = vec![id] }
		
		MeshlessChunk { voxels, pos, additional_data: Addition::Know(data) }
	}

	/// Constructs new empty chunk.
	pub fn new_empty(pos: Int3) -> Self {
		Self { pos, voxels: vec![], additional_data: Addition::Know(AdditionalData { fill: ChunkFill::Empty }) }
	}

	/// Constructs new filled chunk.
	pub fn new_filled(pos: Int3, id: Id) -> Self {
		Self { pos, voxels: vec![id], additional_data: Addition::Know(AdditionalData { fill: ChunkFill::All(id) }) }
	}

	/// Checks if chunk is empty by its data.
	pub fn is_empty(&self) -> bool {
		match self.additional_data {
			Addition::Know(AdditionalData { fill: ChunkFill::Empty }) => true,
			_ => false,
		}
	}

	/// Checks if chunk is empty by its data.
	pub fn is_filled(&self) -> bool {
		match self.additional_data {
			Addition::Know(AdditionalData { fill: ChunkFill::All(_) }) => true,
			_ => false,
		}
	}

	/// Gives fill id if available
	pub fn fill_id(&self) -> Option<Id> {
		if let Addition::Know(AdditionalData { fill: ChunkFill::All(id) }) = self.additional_data {
			Some(id)
		} else { None }
	}

	/// Creates trianlges Vec from Chunk and its environment.
	pub fn to_triangles(&self, env: &ChunkEnvironment) -> Vec<Vertex> {
		match self.additional_data.as_ref() {
			Addition::Know(AdditionalData { fill: ChunkFill::Empty }) => return vec![],
			Addition::Know(AdditionalData { fill: ChunkFill::All(id) }) => {
				/* Cycle over all coordinates in chunk */
				let mut vertices = vec![];
				for x in 0 .. CHUNK_SIZE as i32 {
				for y in 0 .. CHUNK_SIZE as i32 {
				for z in 0 .. CHUNK_SIZE as i32 {
					self.to_triangles_inner(Int3::new(x, y, z), *id, env, &mut vertices);
				}}}

				/* Shrink vector */
				vertices.shrink_to_fit();

				return vertices
			},
			Addition::Know(AdditionalData { fill: ChunkFill::Other }) => {
				/* Construct vertex array */
				let mut vertices = vec![];
				for (i, &voxel_id) in self.voxels.iter().enumerate() {
					self.to_triangles_inner(position_function(i), voxel_id, env, &mut vertices);
				}

				/* Shrink vector */
				vertices.shrink_to_fit();

				return vertices
			},
			Addition::NothingToKnow => panic!(
				"No needed information passed into mesh builder! {:?}",
				Addition::NothingToKnow
			),
		}
	}

	fn to_triangles_inner(&self, in_chunk_pos: Int3, id: Id, env: &ChunkEnvironment, vertices: &mut Vec<Vertex>) {
		if id != NOTHING_VOXEL_DATA.id {
			/* Cube vertices generator */
			let cube = Cube::new(&VOXEL_DATA[id as usize]);

			/* Get position from index */
			let position = pos_in_chunk_to_world(in_chunk_pos, self.pos);

			/* Draw checker */
			let check = |input: Option<Voxel>| -> bool {
				match input {
					None => true,
					Some(voxel) => voxel.data == NOTHING_VOXEL_DATA,
				}
			};

			/* Mesh builder */
			let build = |bias, env: Option<*const MeshlessChunk>| {
				if check(self.get_voxel_or_none(position + bias)) {
					match env {
						Some(chunk) => {
							// * SAFETY: Safe because environment chunks lives as long as other chunks or that given chunk.
							// * And it also needs only at chunk generation stage.
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(position + bias) }) {
								true
							} else { false }
						},
						None => true
					}
				} else { false }
			};

			/* Build all sides separately */
			if build(Int3::new( 1,  0,  0), env.back)   { cube.back  (position, vertices) };
			if build(Int3::new(-1,  0,  0), env.front)  { cube.front (position, vertices) };
			if build(Int3::new( 0,  1,  0), env.top)    { cube.top   (position, vertices) };
			if build(Int3::new( 0, -1,  0), env.bottom) { cube.bottom(position, vertices) };
			if build(Int3::new( 0,  0,  1), env.right)  { cube.right (position, vertices) };
			if build(Int3::new( 0,  0, -1), env.left)   { cube.left  (position, vertices) };
		}
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel_optional(&self, global_pos: Int3) -> FindOptions {
		/* Transform to local */
		let pos = world_coords_to_in_some_chunk(global_pos, self.pos);
		
		if pos.x() < 0 || pos.x() >= CHUNK_SIZE as i32 || pos.y() < 0 || pos.y() >= CHUNK_SIZE as i32 || pos.z() < 0 || pos.z() >= CHUNK_SIZE as i32 {
			FindOptions::Border
		} else {
			match self.additional_data.as_ref() {
				Addition::Know(AdditionalData { fill: ChunkFill::Empty }) =>
					FindOptions::InChunkNothing,
				Addition::Know(AdditionalData { fill: ChunkFill::All(id) }) =>
					FindOptions::InChunkSome(Voxel::new(global_pos, &VOXEL_DATA[*id as usize])),
				_ => {
					/* Sorts: [X -> Y -> Z] */
					let index = index_function(pos);
					FindOptions::InChunkSome(Voxel::new(global_pos, &VOXEL_DATA[self.voxels[index] as usize]))
				}
			}
		}
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
		match self.get_voxel_optional(pos) {
			FindOptions::Border | FindOptions::InChunkNothing => None,
			FindOptions::InChunkSome(chunk) => Some(chunk)
		}
	}

	/// Checks if chunk is in camera view.
	pub fn is_visible(&self, camera: &Camera) -> bool {
		/* AABB init */
		let lo = chunk_cords_to_min_world(self.pos);
		let hi = lo + Int3::all(CHUNK_SIZE as i32);

		/* Frustum check */
		camera.is_aabb_in_view(AABB::from_int3(lo, hi))
	}

	/// Upgrades chunk to be meshed.
	#[allow(dead_code)]
	pub fn envs_upgrade(self, graphics: &Graphics, env: &ChunkEnvironment) -> MeshedChunk {
		MeshedChunk::from_meshless_envs(graphics, self, env)
	}

	/// Upgrades chunk to be meshed.
	pub fn triangles_upgrade(self, graphics: &Graphics, triangles: &[Vertex]) -> MeshedChunk {
		MeshedChunk::from_meshless_triangles(graphics, self, triangles)
	}
}

/// Chunk struct.
pub struct MeshedChunk {
	pub inner: MeshlessChunk,
	pub mesh: RefCell<UnindexedMesh<Vertex>>
}

/// Describes blocked chunks by environent or not. 
#[derive(Clone, Default)]
pub struct ChunkEnvironment<'l> {
	pub top:	Option<*const MeshlessChunk>,
	pub bottom:	Option<*const MeshlessChunk>,
	pub front:	Option<*const MeshlessChunk>,
	pub back:	Option<*const MeshlessChunk>,
	pub left:	Option<*const MeshlessChunk>,
	pub right:	Option<*const MeshlessChunk>,

	pub _phantom: PhantomData<&'l MeshlessChunk>
}

unsafe impl<'c> Send for ChunkEnvironment<'c> { }

impl<'c> ChunkEnvironment<'c> {
	/// Empty description.
	pub fn none() -> Self {
		ChunkEnvironment { top: None, bottom: None, front: None, back: None, left: None, right: None, _phantom: PhantomData }
	}
}

impl MeshedChunk {
	/// Constructs new chunk in given position.
	#[allow(dead_code)]
	pub fn from_envs(graphics: &Graphics, pos: Int3, env: &ChunkEnvironment) -> Self {
		/* Construct new meshless */
		let meshless = MeshlessChunk::new(pos);
		
		/* Upgrade it */
		Self::from_meshless_envs(graphics, meshless, env)
	}

	/// Constructs mesh for meshless chunk.
	#[allow(dead_code)]
	pub fn from_meshless_envs(graphics: &Graphics, meshless: MeshlessChunk, env: &ChunkEnvironment) -> Self {
		/* Create chunk */
		let triangles = meshless.to_triangles(env);

		MeshedChunk {
			inner: meshless,
			mesh: RefCell::new(Self::make_mesh(&graphics.display, &triangles))
		}
	}

	/// Constructs mesh for meshless chunk.
	pub fn from_meshless_triangles(graphics: &Graphics, meshless: MeshlessChunk, triangles: &[Vertex]) -> Self {
		MeshedChunk {
			inner: meshless,
			mesh: RefCell::new(Self::make_mesh(&graphics.display, triangles))
		}
	}

	/// Renders chunk.
	/// * Mesh should be constructed before this function call.
	pub fn render<U: Uniforms>(&self, target: &mut Frame, shader: &Shader, uniforms: &U, draw_params: &glium::DrawParameters, camera: &Camera) -> Result<(), DrawError> {
		/* Borrow mesh */
		let mesh = self.mesh.borrow();

		/* Check if vertex array is empty */
		if !mesh.is_empty() && self.is_visible(camera) {
			mesh.render(target, shader, draw_params, uniforms)
		} else { Ok(()) }
	}

	/// Updates mesh.
	pub fn make_mesh(display: &glium::Display, vertices: &[Vertex]) -> UnindexedMesh<Vertex> {
		/* Vertex buffer for chunks */
		let vertex_buffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);

		Mesh::new(vertex_buffer)
	}

	/// Creates trianlges Vec from Chunk and its environment.
	#[allow(dead_code)]
	pub fn to_triangles(&self, env: &ChunkEnvironment) -> Vec<Vertex> {
		self.inner.to_triangles(env)
	}

	/// Gives voxel by world coordinate.
	#[allow(dead_code)]
	pub fn get_voxel_optional(&self, global_pos: Int3) -> FindOptions {
		self.inner.get_voxel_optional(global_pos)
	}

	/// Gives voxel by world coordinate.
	#[allow(dead_code)]
	pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
		self.inner.get_voxel_or_none(pos)
	}

	/// Checks if chunk is in camera view.
	pub fn is_visible(&self, camera: &Camera) -> bool {
		self.inner.is_visible(camera)
	}
}

unsafe impl StaticSize for VoxelArray {
	fn static_size() -> usize {
		CHUNK_VOLUME * u16::static_size()
	}
}

unsafe impl Reinterpret for MeshlessChunk { }

unsafe impl ReinterpretAsBytes for MeshlessChunk {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let voxels = match self.additional_data {
			Addition::Know(AdditionalData { fill: ChunkFill::Empty }) => {
				Cow::Owned(vec![0; CHUNK_VOLUME])
			},
			Addition::Know(AdditionalData { fill: ChunkFill::All(id) }) => {
				Cow::Owned(vec![id; CHUNK_VOLUME])
			},
			_ => Cow::Borrowed(&self.voxels)
		};

		let mut bytes = Vec::with_capacity(Self::static_size());

		bytes.append(&mut voxels.reinterpret_as_bytes());
		bytes.append(&mut self.pos.reinterpret_as_bytes());

		return bytes
	}
}

unsafe impl ReinterpretFromBytes for MeshlessChunk {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let voxel_array_size: usize = VoxelArray::static_size();

		let voxels = VoxelArray::reinterpret_from_bytes(&source[.. voxel_array_size]);
		let pos = Int3::reinterpret_from_bytes(&source[voxel_array_size .. voxel_array_size + Int3::static_size()]);

		MeshlessChunk { voxels, pos, additional_data: Default::default() }
	}
}

unsafe impl ReinterpretSize for MeshlessChunk {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for MeshlessChunk {
	fn static_size() -> usize {
		VoxelArray::static_size() + Int3::static_size() + 1
	}
}

#[cfg(test)]
mod reinterpret_test {
	use super::*;

	#[test]
	fn reinterpret_chunk() {
		let before = MeshlessChunk::new(Int3::new(12, 12, 11));
		let after = MeshlessChunk::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before.voxels, after.voxels);
		assert_eq!(before.pos, after.pos);
	}
}



/// Transforms world coordinates to chunk 
#[allow(dead_code)]
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

/// Transforms world coordinates to chunk
#[allow(dead_code)]
pub fn world_coords_to_in_chunk(pos: Int3) -> Int3 {
	/* Take voxel coordinates to near-zero */
	let x = pos.x() % CHUNK_SIZE as i32;
	let y = pos.y() % CHUNK_SIZE as i32;
	let z = pos.z() % CHUNK_SIZE as i32;

	/* If negative then convert to positive */

	let x = if x < 0 {
		x + CHUNK_SIZE as i32
	} else { x };

	let y = if y < 0 {
		y + CHUNK_SIZE as i32
	} else { y };

	let z = if z < 0 {
		z + CHUNK_SIZE as i32
	} else { z };

	Int3::new(x, y, z)
}

/// Transforms world coordinates to chunk 
pub fn world_coords_to_in_some_chunk(pos: Int3, chunk: Int3) -> Int3 {
	pos - chunk_cords_to_min_world(chunk)
}

/// Index function
pub fn index_function(pos: Int3) -> usize {
	sdex::get_index(&[pos.x() as usize, pos.y() as usize, pos.z() as usize], &[CHUNK_SIZE; 3])
}

/// Position function
pub fn position_function(i: usize) -> Int3 {
	general_position(i, CHUNK_SIZE, CHUNK_SIZE)
}

/// Position function
pub fn general_position(i: usize, height: usize, depth: usize) -> Int3 {
	let xy = i / depth;

	let z =  i % depth;
	let y = xy % height;
	let x = xy / height;

	Int3::new(x as i32, y as i32, z as i32)
}