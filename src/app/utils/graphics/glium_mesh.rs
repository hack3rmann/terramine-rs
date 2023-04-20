use {
    crate::graphics::glium_shader::Shader,
    glium::{
        Vertex,
        VertexBuffer,
        DrawParameters,
        Surface,
        DrawError,
        uniforms::Uniforms,
        backend::Facade,
        vertex::BufferCreationError,
        index::{NoIndices, PrimitiveType, IndicesSource}
    },
};

pub type UnindexedMesh<V> = Mesh<NoIndices, V>;

/// Handles vertex_buffer and shader.
#[derive(Debug)]
pub struct Mesh<IntoIdx, V: Copy> {
    pub vertices: VertexBuffer<V>,
    pub indices: IntoIdx,
}

impl<'src, IntoIdx, V> Mesh<IntoIdx, V>
where
    IntoIdx: Into<IndicesSource<'src>>,
    V: Vertex,
{
    /// Constructs new mesh.
    pub fn new(vertices: VertexBuffer<V>, indices: IntoIdx) -> Self {
        Self { vertices, indices }
    }

    /// Renders mesh.
    pub fn render<'s>(
        &'s self, target: &mut impl Surface, shader: &Shader,
        draw_params: &DrawParameters<'_>, uniforms: &impl Uniforms
    ) -> Result<(), DrawError>
    where
        &'s IntoIdx: Into<IndicesSource<'src>>,
    {
        target.draw(&self.vertices, &self.indices, &shader.program, uniforms, draw_params)
    }

    /// Checks if vertices vector is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.len() == 0
    }
}

impl<V: Vertex> UnindexedMesh<V> {
    pub fn new_unindexed(vertices: VertexBuffer<V>, primitive_type: PrimitiveType) -> Self {
        Self { vertices, indices: NoIndices(primitive_type) }
    }

    pub fn new_empty(facade: &dyn Facade, primitive_type: PrimitiveType) -> Result<Self, BufferCreationError> {
        let vertices = VertexBuffer::new(facade, &[])?;
        Ok(Self::new_unindexed(vertices, primitive_type))
    }
}