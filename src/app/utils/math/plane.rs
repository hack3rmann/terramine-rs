use crate::app::utils::math::vector::Float4;

/// Represents a plane
#[derive(Clone, Copy)]
pub struct Plane {
	pub normal: Float4,
	pub distance: f32,
}

impl Plane {
	/// Constructs plane from origin and normal
	pub fn from_origin_and_normal(origin: Float4, normal: Float4) -> Self {
		Plane { normal: normal.normalyze(), distance: origin.dot(normal) / normal.abs() }
	}

	/// Checks if gitven vector is in positive side of plane
	pub fn is_in_positive_side(&self, vec: Float4) -> bool {
		self.signed_distance(vec) >= 0.0
	}

	/// Gives signed distance to this plane
	pub fn signed_distance(&self, vec: Float4) -> f32 {
		vec.dot(self.normal) - self.distance
	}
}