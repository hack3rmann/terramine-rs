#![allow(dead_code)]

use super::voxel::{
	Voxel,
	shape
};
use super::voxel::voxel_data::GRASS_VOXEL_DATA;
use crate::app::utils::{
	math::vector::{
		Int3,
		swizzle::*,
	},
	graphics::{
		Graphics,
		mesh::Mesh,
		Vertex,
		shader::Shader,
		vertex_buffer::VertexBuffer
	}
};
use glium::{
	DrawError,
	uniforms::Uniforms,
	Frame
};

/// Predefined chunk values.
const CHUNK_SIZE:	usize = 64;
const CHUNK_VOLUME:	usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Option<Voxel>>;

/// Chunk struct.
pub struct Chunk<'dp> {
	voxels: VoxelArray,
	pos: Int3,
	mesh: Option<Mesh<'dp>>
}

impl<'dp> Chunk<'dp> {
	/// Constructs new chunk in given position 
	pub fn new(graphics: &Graphics, pos: Int3) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(CHUNK_VOLUME);

		/* Iterating in the chunk */
		for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			let global_pos = pos_in_chunk_to_world(Int3::new(x as i32, y as i32, z as i32), pos);
			if global_pos.y() < ((global_pos.x() as f32).sin() * 3.0 + (global_pos.z() as f32).sin() * 3.0 + (global_pos.x() as f32 / 80.0).sin() * 30.0 + (global_pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32 {
				voxels.push(Some(Voxel::new(global_pos, &GRASS_VOXEL_DATA)));
			} else {
			 	voxels.push(None)
			}
		}}}
		
		/* Create chunk */
		let mut chunk = Chunk { voxels, pos, mesh: None };

		/* Create mesh for chunk */
		chunk.update_mesh(graphics);

		return chunk;
	}

	/// Renders chunk.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U) -> Result<(), DrawError> {
		/* Iterating through array */
		self.mesh.as_ref().unwrap().render(target, uniforms)
	}

	/// Updates mesh
	pub fn update_mesh(&mut self, graphics: &Graphics) {
		self.mesh = {
			/* Construct vertex array */
			let mut vertices = Vec::<Vertex>::new();
			for voxel in self.voxels.iter() {
				if let Some(voxel) = voxel {
					/* Top face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() + 1, voxel.position.z())) {
						vertices.append(&mut shape::cube_top(voxel.position));
					}

					/* Bottom face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y() - 1, voxel.position.z())) {
						vertices.append(&mut shape::cube_bottom(voxel.position));
					}
					
					/* Back face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x() + 1, voxel.position.y(), voxel.position.z())) {
						vertices.append(&mut shape::cube_back(voxel.position));
					}
					
					/* Front face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x() - 1, voxel.position.y(), voxel.position.z())) {
						vertices.append(&mut shape::cube_front(voxel.position));
					}
					
					/* Right face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() + 1)) {
						vertices.append(&mut shape::cube_right(voxel.position));
					}
					
					/* Left face check */
					if let None = self.get_voxel_or_none(Int3::new(voxel.position.x(), voxel.position.y(), voxel.position.z() - 1)) {
						vertices.append(&mut shape::cube_left(voxel.position));
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
		};
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

/// Transforms world coordinates to chunk 
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