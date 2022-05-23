/**
 * Vertex buffer wrapper
 */

use crate::graphics::{Graphics, Vertex};

/// Vertex buffer help struct.
pub struct VertexBuffer {
	pub vertex_buffer: glium::VertexBuffer<Vertex>,
	pub indices: glium::index::NoIndices
}

impl VertexBuffer {
	/// Binds vertex buffer to pipeline.
	pub fn bind(self, graphics: &mut Graphics) {
		graphics.upload_vertex_buffer(self);
	}

	/// Test code. Default realisation.
	pub fn default(graphics: &Graphics) -> Self {
		/* Quad vertices */
		let shape = vec! [
			Vertex { position: [-0.9, -0.15 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-0.9,  0.15 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 0.9,  0.15 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-0.9, -0.15 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 0.9,  0.15 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 0.9, -0.15 ], tex_coords: [ 1.0, 0.0 ] }
		];

		/* Define vertex buffer */
		let vertex_buffer = glium::VertexBuffer::new(&graphics.display, &shape).unwrap();
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

		VertexBuffer {
			vertex_buffer: vertex_buffer,
			indices: indices
		}
	}
}