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
		todo!()
	}
}