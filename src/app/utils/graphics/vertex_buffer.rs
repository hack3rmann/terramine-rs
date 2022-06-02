/**
 * Vertex buffer wrapper
 */

use crate::app::graphics::{Graphics, Vertex};

/// Vertex buffer help struct.
pub struct VertexBuffer {
	pub vertex_buffer: glium::VertexBuffer<Vertex>,
	pub indices: glium::index::NoIndices
}

impl VertexBuffer {
	/// Test code. Default realisation.
	pub fn default(graphics: &Graphics) -> Self {
		/* Quad vertices */
		let shape = vec! [
			/* Front */
			Vertex { position: [-1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-1.0, -1.0,  1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0, -1.0 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			/* Back */
			Vertex { position: [ 1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 1.0, -1.0,  1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 1.0,  1.0, -1.0 ], tex_coords: [ 1.0, 0.0 ] },
			/* Left */
			Vertex { position: [ 1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 1.0,  1.0, -1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-1.0,  1.0, -1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0, -1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-1.0, -1.0, -1.0 ], tex_coords: [ 1.0, 0.0 ] },
			/* Right */
			Vertex { position: [ 1.0, -1.0,  1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 1.0,  1.0,  1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 1.0, -1.0,  1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0, -1.0,  1.0 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			/* Up */
			Vertex { position: [ 1.0,  1.0, -1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [ 1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-1.0,  1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0,  1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [-1.0,  1.0,  1.0 ], tex_coords: [ 1.0, 0.0 ] },
			/* Up */
			Vertex { position: [-1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [ 1.0, -1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
			Vertex { position: [ 1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 1.0 ] },
			Vertex { position: [-1.0, -1.0, -1.0 ], tex_coords: [ 0.0, 0.0 ] },
			Vertex { position: [-1.0, -1.0,  1.0 ], tex_coords: [ 1.0, 0.0 ] },
			Vertex { position: [ 1.0, -1.0,  1.0 ], tex_coords: [ 1.0, 1.0 ] },
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