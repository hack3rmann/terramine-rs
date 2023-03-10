/**
 * Vertex buffer wrapper
 */

use {
    glium::{
        index::{IndicesSource, PrimitiveType, NoIndices},
        VertexBuffer as GVertexBuffer,
        Vertex as TVertex,
    },
};

/// Vertex buffer help struct.
#[derive(Debug)]
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
    pub fn new(facade: &dyn glium::backend::Facade, vertices: &[Vertex], indices: Idx) -> Self {
        /* Define vertex buffer */
        let vertex_buffer = GVertexBuffer::new(facade, vertices)
            .expect("failed to create new vertex buffer");

        VertexBuffer { inner: vertex_buffer, indices }
    }
}

impl<Vertex: Copy + TVertex> VertexBuffer<NoIndices, Vertex> {
    /// Constructs new [`VertexBuffer`] from vertices and [`PrimitiveType`].
    pub fn no_indices(facade: &dyn glium::backend::Facade, vertices: &[Vertex], primitive_type: PrimitiveType) -> Self {
        Self::new(facade, vertices, NoIndices(primitive_type))
    }

    /// Constructs empty vertex buffer.
    pub fn new_empty(facade: &dyn glium::backend::Facade) -> Self {
        /* Define vertex buffer */
        let vertex_buffer = GVertexBuffer::new(facade, &[])
            .expect("failed to create new vertex buffer");
        Self { inner: vertex_buffer, indices: NoIndices(PrimitiveType::Points) }
    }
}