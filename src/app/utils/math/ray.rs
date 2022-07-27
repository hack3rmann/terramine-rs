#![allow(dead_code)]

use super::prelude::*;

#[cfg(not(feature = "release"))]
use crate::app::utils::graphics::{
	mesh::{Mesh, UnindexedMesh},
	Graphics,
	vertex_buffer::VertexBuffer,
	Vertex,
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
	pub fn get_mesh(self, graphics: &Graphics) -> UnindexedMesh {
		use glium::index::PrimitiveType;
		
		let far = (self.origin + self.direction) * 100.0;

		let vertices = vec![
			Vertex { position: [self.origin.x(), self.origin.y(), self.origin.z()], tex_coords: [0.0, 0.0], light: 1.0 },
			Vertex { position: [        far.x(),         far.y(),         far.z()], tex_coords: [0.0, 0.0], light: 1.0 },
		];
		
		/* Vertex buffer for chunks */
		let vertex_buffer = VertexBuffer::no_indices(&graphics.display, &vertices, PrimitiveType::TrianglesList);

		Mesh::new(vertex_buffer)
	}
}