pub mod voxel_data;

use voxel_data::*;
use crate::app::utils::{
	math::vector::*,
	graphics::{
		Graphics,
		mesh::Mesh,
		vertex_buffer::VertexBuffer,
		shader::Shader
	},
};

#[allow(dead_code)]
pub struct Voxel<'v> {
	pub data: &'static VoxelData,
	pub position: Int3,

	/* TEMPORARY */
	pub mesh: Mesh<'v>
}

impl<'v> Voxel<'v> {
	pub fn default(graphics: &Graphics) -> Self {
		let mesh = {
			let draw_params = glium::DrawParameters {
				depth: glium::Depth {
					test: glium::DepthTest::IfLess,
					write: true,
					.. Default::default()
				},
				backface_culling: glium::BackfaceCullingMode::CullClockwise,
				.. Default::default()
			};
			let shader = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
			Mesh::new(VertexBuffer::default(graphics), shader, draw_params)
		};

		Voxel {
			data: &FIRST_VOXEL_DATA,
			position: Default::default(),
			mesh: mesh
		}
	}
	
	pub fn new(position: Int3, data: &'static VoxelData) -> Self {
		
	}
}