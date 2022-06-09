pub mod chunk_array;

use super::voxel::{
	Voxel,
	shape::Cube,
	voxel_data::{LOG_VOXEL_DATA, STONE_VOXEL_DATA},
};
use crate::app::utils::{
	math::prelude::*,
	graphics::{
		Graphics,
		mesh::Mesh,
		Vertex,
		shader::Shader,
		vertex_buffer::VertexBuffer,
		camera::Camera,
	}
};
use glium::{
	DrawError,
	uniforms::Uniforms,
	Frame
};
use std::cell::RefCell;

/// Predefined chunk values.
const CHUNK_SIZE:	usize = 64;
const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Option<Voxel>>;

/// Chunk struct.
pub struct Chunk<'dp> {
	voxels: VoxelArray,
	pos: Int3,
	mesh: RefCell<Option<Mesh<'dp>>>
}

/// Describes blocked chunks by environent or not. 
#[derive(Clone, Default)]
pub struct ChunkEnvironment<'c> {
	pub top:	Option<*const Chunk<'c>>,
	pub bottom:	Option<*const Chunk<'c>>,
	pub front:	Option<*const Chunk<'c>>,
	pub back:	Option<*const Chunk<'c>>,
	pub left:	Option<*const Chunk<'c>>,
	pub right:	Option<*const Chunk<'c>>,
}

impl<'c> ChunkEnvironment<'c> {
	/// Empty description.
	pub fn none() -> Self {
		ChunkEnvironment { top: None, bottom: None, front: None, back: None, left: None, right: None }
	}
}

impl<'dp> Chunk<'dp> {
	/// Constructs new chunk in given position 
	pub fn new(graphics: &Graphics, pos: Int3, generate_mesh: bool) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(CHUNK_VOLUME);

		/* Iterating in the chunk */
		for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			let global_pos = pos_in_chunk_to_world(Int3::new(x as i32, y as i32, z as i32), pos);
			if global_pos.y() < ((global_pos.x() as f32).sin() * 3.0 + (global_pos.z() as f32).sin() * 3.0 + (global_pos.x() as f32 / 80.0).sin() * 30.0 + (global_pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32 &&
			   global_pos.y() >= ((global_pos.x() as f32 / 80.0).sin() * 30.0 + (global_pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32
			{
				voxels.push(Some(Voxel::new(global_pos, &LOG_VOXEL_DATA)));
			}
			else if global_pos.y() < ((global_pos.x() as f32 / 80.0).sin() * 30.0 + (global_pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32 {
				voxels.push(Some(Voxel::new(global_pos, &STONE_VOXEL_DATA)))
			} else {
			 	voxels.push(None)
			}
		}}}
		
		/* Create chunk */
		let chunk = Chunk { voxels, pos, mesh: RefCell::new(None) };

		/* Create mesh for chunk */
		if generate_mesh {
			chunk.update_mesh(graphics, &ChunkEnvironment::none());
		}

		return chunk;
	}

	/// Renders chunk.
	/// * Mesh should be constructed before this function call.
	pub fn render<U: Uniforms>(&self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		let mesh = self.mesh.borrow();
		let mesh = mesh.as_ref().unwrap();

		/* Check if vertex array is empty */
		if !mesh.is_empty() && self.is_visible(camera) {
			/* Iterating through array */
			mesh.render(target, uniforms)
		} else {
			Ok(( ))
		}
	}

	/// Updates mesh
	pub fn update_mesh(&self, graphics: &Graphics, env: &ChunkEnvironment) {
		self.mesh.replace({
			/* Construct vertex array */
			let mut vertices = Vec::<Vertex>::new();
			for voxel in self.voxels.iter() {
				if let Some(voxel) = voxel {
					/*
					 * Safe because environment chunks lives as long as other chunks or that given chunk.
					 * And it also needs only at chunk generation stage.
					 */

					/* Cube vertices generator */
					let cube = Cube::new(voxel.data);

					/* Top face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() + 1, voxel.position.z())) {
						match env.top {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() + 1, voxel.position.z())) } {
									vertices.append(&mut cube.top(voxel.position))
								}
							},
							None => vertices.append(&mut cube.top(voxel.position))
						}
					}

					/* Bottom face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() - 1, voxel.position.z())) {
						match env.bottom {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() - 1, voxel.position.z())) } {
									vertices.append(&mut cube.bottom(voxel.position))
								}
							},
							None => vertices.append(&mut cube.bottom(voxel.position))
						}
					}
					
					/* Back face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x() + 1, voxel.position.y(), voxel.position.z())) {
						match env.back {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x() + 1, voxel.position.y(), voxel.position.z())) } {
									vertices.append(&mut cube.back(voxel.position))
								}
							},
							None => vertices.append(&mut cube.back(voxel.position))
						}
					}
					
					/* Front face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x() - 1, voxel.position.y(), voxel.position.z())) {
						match env.front {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x() - 1, voxel.position.y(), voxel.position.z())) } {
									vertices.append(&mut cube.front(voxel.position))
								}
							},
							None => vertices.append(&mut cube.front(voxel.position))
						}
					}
					
					/* Right face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() + 1)) {
						match env.right {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() + 1)) } {
									vertices.append(&mut cube.right(voxel.position))
								}
							},
							None => vertices.append(&mut cube.right(voxel.position))
						}
					}
					
					/* Left face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() - 1)) {
						match env.left {
							Some(chunk) => {
								if let None = unsafe { chunk.as_ref().unwrap().get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() - 1)) } {
									vertices.append(&mut cube.left(voxel.position))
								}
							},
							None => vertices.append(&mut cube.left(voxel.position))
						}
					}
				}
			}

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
			
			/* Shader for chunks */
			let shader = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
			
			/* Vertex buffer for chunks */
			let vertex_buffer = VertexBuffer::from_vertices(graphics, vertices);
	
			Some(Mesh::new(vertex_buffer, shader, draw_params))
		});
	}

	/// Gives voxel by world coordinate
	pub fn get_voxel_or_none(&self, pos: Int3) -> Option<&Voxel> {
		/* Transform to local */
		let pos = world_coords_to_in_some_chunk(pos, self.pos);
		
		if pos.x() < 0 || pos.x() >= CHUNK_SIZE as i32 || pos.y() < 0 || pos.y() >= CHUNK_SIZE as i32 || pos.z() < 0 || pos.z() >= CHUNK_SIZE as i32 {
			None
		} else {
			/* Sorts: [X -> Y -> Z] */
			let index = (pos.x() * CHUNK_SIZE as i32 + pos.y()) * CHUNK_SIZE as i32 + pos.z();
			(&self.voxels[index as usize]).as_ref()
		}
	}

	/// Checks if chunk is in camera view
	pub fn is_visible(&self, camera: &Camera) -> bool {
		/* AABB init */
		let lo = chunk_cords_to_min_world(self.pos);
		let hi = lo + Int3::all(CHUNK_SIZE as i32);

		/* Frustum check */
		camera.is_aabb_in_view(AABB::from_int3(lo, hi))
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