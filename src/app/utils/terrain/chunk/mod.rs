pub mod chunk_array;

use {
	crate::app::utils::{
		werror::prelude::*,
		math::prelude::*,
		graphics::{
			Graphics,
			mesh::Mesh,
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
	},
	std::{cell::RefCell, marker::PhantomData},
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
const CHUNK_SIZE:	usize = 64;
const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<u16>;

#[allow(dead_code)]
pub enum FindOptions {
	Border,
	InChunkNothing,
	InChunkSome(Voxel)
}

/// Chunk struct.
pub struct MeshlessChunk {
	pub voxels: VoxelArray,
	pub pos: Int3,
}

impl MeshlessChunk {
	/// Constructs new chunk in given position 
	pub fn new(pos: Int3) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(CHUNK_VOLUME);

		/* Iterating in the chunk */
		for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			let global_pos = pos_in_chunk_to_world(Int3::new(x as i32, y as i32, z as i32), pos);

			/* Kind of trees generation */
			if generator::trees(global_pos) {
				voxels.push(LOG_VOXEL_DATA.id);
			}
			
			/* Sine-like floor */
			else if generator::sine(global_pos) {
				voxels.push(STONE_VOXEL_DATA.id);
			}
			
			/* Air */
			else {
			 	voxels.push(NOTHING_VOXEL_DATA.id)
			}
		}}}
		
		MeshlessChunk { voxels, pos }
	}

	/// Creates trianlges Vec from Chunk and its environment.
	pub fn to_triangles(&self, env: &ChunkEnvironment) -> Vec<Vertex> {
		/* Construct vertex array */
		let mut vertices = Vec::<Vertex>::new();
		for (i, &voxel_id) in self.voxels.iter().enumerate() {
			if voxel_id != NOTHING_VOXEL_DATA.id {
				/*
				 * Safe because environment chunks lives as long as other chunks or that given chunk.
				 * And it also needs only at chunk generation stage.
				 */

				/* Cube vertices generator */
				let cube = Cube::new(&VOXEL_DATA[voxel_id as usize]);

				/* Get position from index */
				let position = pos_in_chunk_to_world(position_function(i), self.pos);

				/* Draw checker */
				let check = |input: Option<Voxel>| -> bool {
					match input {
						None => true,
						Some(voxel) => voxel.data == NOTHING_VOXEL_DATA,
					}
				};

				/* Top face check */
				if check(self.get_voxel_or_none(Int3::new(position.x(), position.y() + 1, position.z()))) {
					match env.top {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x(), position.y() + 1, position.z())) }) {
								cube.top(position, &mut vertices)
							}
						},
						None => cube.top(position, &mut vertices)
					}
				}

				/* Bottom face check */
				if check(self.get_voxel_or_none(Int3::new(position.x(), position.y() - 1, position.z()))) {
					match env.bottom {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x(), position.y() - 1, position.z())) }) {
								cube.bottom(position, &mut vertices)
							}
						},
						None => cube.bottom(position, &mut vertices)
					}
				}
				
				/* Back face check */
				if check(self.get_voxel_or_none(Int3::new(position.x() + 1, position.y(), position.z()))) {
					match env.back {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x() + 1, position.y(), position.z())) }) {
								cube.back(position, &mut vertices)
							}
						},
						None => cube.back(position, &mut vertices)
					}
				}
				
				/* Front face check */
				if check(self.get_voxel_or_none(Int3::new(position.x() - 1, position.y(), position.z()))) {
					match env.front {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x() - 1, position.y(), position.z())) }) {
								cube.front(position, &mut vertices)
							}
						},
						None => cube.front(position, &mut vertices)
					}
				}
				
				/* Right face check */
				if check(self.get_voxel_or_none(Int3::new(position.x(), position.y(), position.z() + 1))) {
					match env.right {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x(), position.y(), position.z() + 1)) }) {
								cube.right(position, &mut vertices)
							}
						},
						None => cube.right(position, &mut vertices)
					}
				}
				
				/* Left face check */
				if check(self.get_voxel_or_none(Int3::new(position.x(), position.y(), position.z() - 1))) {
					match env.left {
						Some(chunk) => {
							if check(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(Int3::new(position.x(), position.y(), position.z() - 1)) }) {
								cube.left(position, &mut vertices)
							}
						},
						None => cube.left(position, &mut vertices)
					}
				}
			}
		}

		/* Shrink vector */
		vertices.shrink_to_fit();

		return vertices
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel_optional(&self, global_pos: Int3) -> FindOptions {
		/* Transform to local */
		let pos = world_coords_to_in_some_chunk(global_pos, self.pos);
		
		if pos.x() < 0 || pos.x() >= CHUNK_SIZE as i32 || pos.y() < 0 || pos.y() >= CHUNK_SIZE as i32 || pos.z() < 0 || pos.z() >= CHUNK_SIZE as i32 {
			FindOptions::Border
		} else {
			/* Sorts: [X -> Y -> Z] */
			let index = index_function(pos);
			FindOptions::InChunkSome(Voxel::new(global_pos, &VOXEL_DATA[self.voxels[index] as usize]))
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
	pub fn envs_upgrade<'dp, 'g: 'dp>(self, graphics: &'g Graphics, env: &ChunkEnvironment) -> MeshedChunk<'dp> {
		MeshedChunk::from_meshless_envs(graphics, self, env)
	}

	/// Upgrades chunk to be meshed.
	pub fn triangles_upgrade<'dp, 'g>(self, graphics: &'g Graphics, triangles: Vec<Vertex>) -> MeshedChunk<'dp> {
		MeshedChunk::from_meshless_triangles(graphics, self, triangles)
	}
}

/// Chunk struct.
pub struct MeshedChunk<'dp> {
	inner: MeshlessChunk,
	mesh: RefCell<Mesh<'dp>>
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

impl<'dp> MeshedChunk<'dp> {
	/// Constructs new chunk in given position.
	#[allow(dead_code)]
	pub fn from_envs<'g>(graphics: &'g Graphics, pos: Int3, env: &ChunkEnvironment) -> Self {
		/* Construct new meshless */
		let meshless = MeshlessChunk::new(pos);
		
		/* Upgrade it */
		Self::from_meshless_envs(graphics, meshless, env)
	}

	/// Constructs mesh for meshless chunk.
	#[allow(dead_code)]
	pub fn from_meshless_envs<'g>(graphics: &'g Graphics, meshless: MeshlessChunk, env: &ChunkEnvironment) -> Self {
		/* Create chunk */
		let triangles = meshless.to_triangles(env);

		MeshedChunk {
			inner: meshless,
			mesh: RefCell::new(Self::make_mesh(&graphics.display, triangles))
		}
	}

	/// Constructs mesh for meshless chunk.
	pub fn from_meshless_triangles<'g>(graphics: &'g Graphics, meshless: MeshlessChunk, triangles: Vec<Vertex>) -> Self {
		MeshedChunk {
			inner: meshless,
			mesh: RefCell::new(Self::make_mesh(&graphics.display, triangles))
		}
	}

	/// Renders chunk.
	/// * Mesh should be constructed before this function call.
	pub fn render<U: Uniforms>(&self, target: &mut Frame, shader: &Shader, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		/* Borrow mesh */
		let mesh = self.mesh.borrow();

		/* Check if vertex array is empty */
		if !mesh.is_empty() && self.is_visible(camera) {
			/* Iterating through array */
			mesh.render(target, shader, uniforms)
		} else {
			Ok(())
		}
	}

	/// Updates mesh.
	pub fn make_mesh<'d>(display: &'d glium::Display, vertices: Vec<Vertex>) -> Mesh<'dp> {
		/* Chunk draw parameters */
		let draw_params = glium::DrawParameters {
			depth: glium::Depth {
				test: glium::DepthTest::IfLess,
				write: true,
				.. Default::default()
			},
			backface_culling: glium::BackfaceCullingMode::CullClockwise,
			.. Default::default()
		};
		
		/* Vertex buffer for chunks */
		let vertex_buffer = VertexBuffer::from_vertices(display, vertices);

		Mesh::new(vertex_buffer, draw_params)
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
		let mut bytes = Vec::with_capacity(Self::static_size());

		bytes.append(&mut self.voxels.reinterpret_as_bytes());
		bytes.append(&mut self.pos.reinterpret_as_bytes());

		return bytes;
	}
}

unsafe impl ReinterpretFromBytes for MeshlessChunk {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let voxel_array_size: usize = VoxelArray::static_size();

		let voxels = VoxelArray::reinterpret_from_bytes(&source[.. voxel_array_size]);
		let pos = Int3::reinterpret_from_bytes(&source[voxel_array_size .. voxel_array_size + Int3::static_size()]);

		MeshlessChunk { voxels, pos }
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