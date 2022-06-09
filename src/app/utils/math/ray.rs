#![allow(dead_code)]

use super::prelude::*;

/// Represents mathematical ray
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
}