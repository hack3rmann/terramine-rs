#![allow(dead_code)]

use super::prelude::*;

use crate::app::utils::graphics::{
	mesh::Mesh,
	Graphics,
	shader::Shader,
	vertex_buffer::VertexBuffer, Vertex
};

/// Represents mathematical ray
#[derive(Clone, Copy)]
pub struct Ray {
	pub origin: Float4,
	pub direction: Float4,
}

impl Ray {
	/// Creates new ray
    pub fn new(origin: Float4, direction: Float4) -> Self { Ray { origin, direction } }

	/// Creates ray from 2 points
	pub fn from_2_points(start: Float4, end: Float4) -> Self {
		Ray { origin: start, direction: (end - start).normalyze() }
	}

	/// Gives temporary mesh. Good for debugging
	#[allow(dead_code)]
	#[cfg(not(feature = "release"))]
	pub fn get_mesh<'g, 'm>(self, graphics: &'g Graphics) -> Mesh<'m> {
		let far = (self.origin + self.direction) * 100.0;

		let vertices = vec![
			Vertex { position: [self.origin.x(), self.origin.y(), self.origin.z()], tex_coords: [0.0, 0.0], light: 1.0 },
			Vertex { position: [        far.x(),         far.y(),         far.z()], tex_coords: [0.0, 0.0], light: 1.0 },
		];
			
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
		let vertex_buffer = VertexBuffer::from_vertices(&graphics.display, vertices);

		Mesh::new(vertex_buffer, shader, draw_params)
	}
}