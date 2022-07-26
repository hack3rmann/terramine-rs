/**
 * Vertex buffer wrapper
 */

use crate::app::{
	utils::werror::prelude::*,
	graphics::Vertex,
};

/// Vertex buffer help struct.
pub struct VertexBuffer {
	pub vertex_buffer: glium::VertexBuffer<Vertex>,
	pub indices: glium::index::NoIndices
}

impl VertexBuffer {
	/// Constructs vertex buffer from vertices vector.
	pub fn from_vertices(display: &glium::Display, vertices: Vec<Vertex>) -> Self {
		/* Define vertex buffer */
		let vertex_buffer = glium::VertexBuffer::new(display, &vertices).wunwrap();
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

		VertexBuffer { vertex_buffer, indices }
	}
}