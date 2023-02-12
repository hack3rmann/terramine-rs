use {
    crate::app::utils::graphics::{
        VertexBuffer,
        Shader
    },
    glium::{
        Vertex as TVertex,
        Display,
        DrawParameters,
        Frame,
        Surface,
        DrawError,
        uniforms::Uniforms,
        index::{NoIndices, IndicesSource}
    },
};

pub type UnindexedMesh<Vertex> = Mesh<NoIndices, Vertex>;

/// Handles vertex_buffer and shader.
pub struct Mesh<Idx, Vertex: Copy> {
    vertex_buffer: VertexBuffer<Idx, Vertex>,
}

impl<Idx, Vertex: Copy> Mesh<Idx, Vertex> {
    /// Constructs new mesh.
    pub fn new(vertex_buffer: VertexBuffer<Idx, Vertex>) -> Self {
        Mesh { vertex_buffer }
    }

    /// Renders mesh.
    pub fn render<'a, U>(&'a self, target: &mut Frame, shader: &Shader, draw_params: &DrawParameters<'_>, uniforms: &U) -> Result<(), DrawError>
    where
        U: Uniforms,
        &'a Idx: Into<IndicesSource<'a>>,
    {
        target.draw(&self.vertex_buffer.inner, &self.vertex_buffer.indices, &shader.program, uniforms, draw_params)
    }

    /// Checks if vertices vector is empty
    pub fn is_empty(&self) -> bool {
        self.vertex_buffer.inner.len() == 0
    }
}

impl <Vertex: Copy + TVertex> Mesh<NoIndices, Vertex> {
    pub fn new_empty(display: &Display) -> Self {
        Mesh { vertex_buffer: VertexBuffer::new_empty(display) }
    }
}