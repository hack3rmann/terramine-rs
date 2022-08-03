//! A bunch of handy iterators for chunks.

use {
	crate::app::utils::{
		math::prelude::*,
	},
};

/// Iterator over chunk border.
#[derive(Clone, Debug)]
pub struct CubeBorder {
	prev: Int3,
	size: i32,
}

impl CubeBorder {
	const INITIAL_STATE: i32 = -1;
	pub fn new(size: i32) -> Self { Self { prev: Int3::all(Self::INITIAL_STATE), size } }
}

impl Iterator for CubeBorder {
	type Item = Int3;
	fn next(&mut self) -> Option<Self::Item> {
		/* Maximun corrdinate value in border */
		let max: i32 = self.size - 1;
		let max_minus_one: i32 = max - 1;

		/* Return if maximum reached */
		if self.prev == Int3::new(max, max, max) {
			return None
		} else if self.prev == Int3::all(Self::INITIAL_STATE) {
			let new = Int3::all(0);
			self.prev = new;
			return Some(new)
		}

		/* Previous x, y and z */
		let (x, y, z) = (self.prev.x(), self.prev.y(), self.prev.z());

		/* Returning local function */
		let mut give = |pos| {
			self.prev = pos;
			return Some(pos)
		};

		/* If somewhere slicing cube (in 1 .. MAX - 1) */
		if x >= 1 && x <= max_minus_one {
			/* If position is slicing square */
			if y >= 1 && y <= max_minus_one  {
				if z == 0 { give(Int3::new(x, y, max)) }
				else if z == max { give(Int3::new(x, y + 1, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0 or {max}! But actual value is {y}.",
					max = max,
					y = y,
				)}
			}

			/* If position is on flat bottom of square */
			else if y == 0 {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(x, y + 1, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* If position is on flat top of square */
			else if y == max {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(x + 1, 0, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* Other Ys are unreachable */
			else { unreachable!(
				"Invalid y position! Must be in 0..{size}! But actual value is {y}.",
				size = self.size,
				y = y,
			)}
		}

		/* If current position is bottom */
		else if x == 0 {
			/* Y is not maximum */
			if y >= 0 && y <= max_minus_one {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(x, y + 1, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* Y is maximum */
			else if y == max {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(x + 1, 0, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* Others Ys are unreachable */
			else { unreachable!(
				"Invalid y position! Must be in 0..{size}! But actual value is {y}.",
				size = self.size,
				y = y,
			)}
		}

		/* If currents position is top */
		else if x == max {
			/* Y is not maximum */
			if y >= 0 && y <= max_minus_one {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(x, y + 1, 0)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* Y is maximum */
			else if y == max {
				if z >= 0 && z <= max_minus_one { give(Int3::new(x, y, z + 1)) }
				else if z == max { give(Int3::new(max, max, max)) }
				else { unreachable!(
					"Invalid z position! Must be in 0..{size}! But actual value is {z}.",
					size = self.size,
					z = z,
				)}
			}

			/* Others Ys are unreachable */
			else { unreachable!(
				"Invalid y position! Must be in 0..{size}! But actual value is {y}.",
				size = self.size,
				y = y,
			)}
		}

		/* Other values of X is unreachable... */
		else { unreachable!(
			"Invalid x position! Must be in 0..{size}! But actual value is {x}.",
			size = self.size,
			x = x,
		)}
	}
}

pub struct SpaceIter {
	curr: Int3,

	min: Int3,
	max: Int3,
}

impl SpaceIter {
	pub fn new(range: std::ops::Range<Int3>) -> Self {
		Self { curr: range.start, min: range.start, max: range.end - Int3::unit() }
	}
}

impl Iterator for SpaceIter {
	type Item = Int3;
	fn next(&mut self) -> Option<Self::Item> {
		let result = self.curr;

		if self.curr.z() < self.max.z() {
			self.curr += Int3::new(0, 0, 1);
		} else {
			if self.curr.y() < self.max.y() {
				self.curr = Int3::new(self.curr.x(), self.curr.y() + 1, self.min.z());
			} else {
				if self.curr.x() < self.max.x() {
					self.curr = Int3::new(self.curr.x() + 1, self.min.y(), self.min.z());
				} else if self.curr == self.max {
					self.curr = self.max + Int3::unit();
				} else if self.curr == self.max + Int3::unit() {
					return None
				}
			}
		}

		return Some(result)
	}
}