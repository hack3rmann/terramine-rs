/**
 * Vertex buffer wrapper
 */

use crate::app::{
	utils::werror::prelude::*,
	glium::{
		index::{IndicesSource, PrimitiveType, NoIndices},
		VertexBuffer as GVertexBuffer,
		Display,
		Vertex as TVertex,
	},
};

/// Vertex buffer help struct.
pub struct VertexBuffer<Idx, Vertex: Copy> {
	pub inner: GVertexBuffer<Vertex>,
	pub indices: Idx
}

impl<'src, Idx, Vertex> VertexBuffer<Idx, Vertex>
where
	Idx: Into<IndicesSource<'src>>,
	Vertex: Copy + TVertex,
{
	/// Constructs [`VertexBuffer`] from vertex vector.
	pub fn new(display: &Display, vertices: &[Vertex], indices: Idx) -> Self {
		/* Define vertex buffer */
		let vertex_buffer = GVertexBuffer::new(display, vertices).wunwrap();

		VertexBuffer { inner: vertex_buffer, indices }
	}
}

impl<Vertex: Copy + TVertex> VertexBuffer<NoIndices, Vertex> {
	/// Constructs new [`VertexBuffer`] from vertices and [`PrimitiveType`].
	pub fn no_indices(display: &Display, vertices: &[Vertex], primitive_type: PrimitiveType) -> Self {
		Self::new(display, vertices, NoIndices(primitive_type))
	}
}