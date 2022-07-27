use {
	crate::app::utils::graphics::{
		VertexBuffer,
		Shader
	},
	glium::{
		DrawParameters,
		Frame,
		Surface,
		DrawError,
		uniforms::Uniforms
	},
};

/// Handles vertex_buffer and shader.
pub struct Mesh {
	vertex_buffer: VertexBuffer,
}

impl Mesh {
	/// Constructs new mesh.
	pub fn new(vertex_buffer: VertexBuffer) -> Self {
		Mesh { vertex_buffer }
	}

	/// Renders mesh.
	pub fn render<U: Uniforms>(&self, target: &mut Frame, shader: &Shader, draw_params: &DrawParameters<'_>, uniforms: &U) -> Result<(), DrawError> {
		target.draw(&self.vertex_buffer.vertex_buffer, &self.vertex_buffer.indices, &shader.program, uniforms, draw_params)
	}

	/// Checks if vertices vector is empty
	pub fn is_empty(&self) -> bool {
		self.vertex_buffer.vertex_buffer.len() == 0
	}
}