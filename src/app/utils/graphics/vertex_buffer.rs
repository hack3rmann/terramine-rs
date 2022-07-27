/**
 * Vertex buffer wrapper
 */

use crate::app::{
	utils::werror::prelude::*,
	graphics::Vertex,
	glium::{
		index::{IndicesSource, PrimitiveType, NoIndices},
		VertexBuffer as GVertexBuffer,
		Display,
	},
};

/// Vertex buffer help struct.
pub struct VertexBuffer<Idx> {
	pub inner: GVertexBuffer<Vertex>,
	pub indices: Idx
}

impl<'a, Idx: Into<IndicesSource<'a>>> VertexBuffer<Idx> {
	/// Constructs [`VertexBuffer`] from vertex vector.
	#[allow(dead_code)]
	pub fn new(display: &Display, vertices: Vec<Vertex>, indices: Idx) -> Self {
		/* Define vertex buffer */
		let vertex_buffer = GVertexBuffer::new(display, &vertices).wunwrap();

		VertexBuffer { inner: vertex_buffer, indices }
	}
}

impl VertexBuffer<NoIndices> {
	/// Constructs new [`VertexBuffer`] from vertices and [`PrimitiveType`].
	pub fn no_indices(display: &Display, vertices: Vec<Vertex>, primitive_type: PrimitiveType) -> Self {
		/* Define vertex buffer */
		let vertex_buffer = GVertexBuffer::new(display, &vertices).wunwrap();

		VertexBuffer { inner: vertex_buffer, indices: NoIndices(primitive_type) }
	}
}