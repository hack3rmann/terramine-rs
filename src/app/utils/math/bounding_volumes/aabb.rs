use crate::app::utils::{
	math::prelude::*,
	graphics::camera::frustum::Frustum,
};

/// Represents axis aligned bounding box
#[derive(Clone, Copy)]
pub struct AABB {
	lo: Float4,
	hi: Float4,
}

impl AABB {
	/// Constructs AABB from Float4 vectors
	#[allow(dead_code)]
	pub fn from_float4(lo: Float4, hi: Float4) -> Self { AABB { lo, hi } }

	/// Constructs AABB from Int3 vectors
	pub fn from_int3(lo: Int3, hi: Int3) -> Self {
		AABB {
			lo: Float4::xyz1(lo.x() as f32, lo.y() as f32, lo.z() as f32),
			hi: Float4::xyz1(hi.x() as f32, hi.y() as f32, hi.z() as f32),
		}
	}

	/// Frustum check
	pub fn is_in_frustum(self, frustum: &Frustum) -> bool {
		/* Frirst pass]
		 * Checks if center of chunk is in frustum
		 * Very cheap operation */
		if frustum.is_in_frustum(self.center()) {
			return true;
		}

		/* Second pass
		 * Checks every vertex of AABB is behind the frustum
		 ? 8 times more expensive than previous */

		let vertex_set = self.as_vertex_set();

		let mut result = false;
		for vertex in vertex_set {
			if frustum.near.is_in_positive_side(vertex) {
				result = true;
				break;
			}
		}
		if !result { return result }

		/* Third pass
		 * Checks every vertex of AABB is in frustum
		 ? 6 times more expensive than previous */

		for vertex in vertex_set {
			if frustum.is_in_frustum(vertex) {
				return true;
			}
		}

		/* Fourth pass
		 * Checks if someone of 4 frustum corner rays intersects AABB
		 ! Very expensive! */
		
		

		/* All passed */
		return false;
	}

	/// Represents AABB as vertex set
	pub fn as_vertex_set(self) -> [Float4; 8] {
		[
			Float4::xyz1(self.lo.x(), self.lo.y(), self.lo.z()),
			Float4::xyz1(self.lo.x(), self.lo.y(), self.hi.z()),
			Float4::xyz1(self.lo.x(), self.hi.y(), self.lo.z()),
			Float4::xyz1(self.lo.x(), self.hi.y(), self.hi.z()),
			Float4::xyz1(self.hi.x(), self.lo.y(), self.lo.z()),
			Float4::xyz1(self.hi.x(), self.lo.y(), self.hi.z()),
			Float4::xyz1(self.hi.x(), self.hi.y(), self.lo.z()),
			Float4::xyz1(self.hi.x(), self.hi.y(), self.hi.z()),
		]
	}

	/// Gives center point in AABB
	pub fn center(self) -> Float4 {
		(self.lo + self.hi) / 2.0
	}
}