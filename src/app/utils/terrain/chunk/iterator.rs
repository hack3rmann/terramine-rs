//! A bunch of handy iterators for chunks.

use {
	crate::app::utils::{
		math::prelude::*,
		werror::prelude::*,
	},
	std::ops::Range,
};

/// Iterator over chunk border.
/// 
/// # Example:
/// ```
/// use crate::app::utils::terrain::chunk::{
/// 	position_function,
/// 	iterator::CubeBorder,
/// };
/// 
/// fn test2() {
/// 	/* [`CubeBorder`] iterator */
/// 	let border = CubeBorder::new(16);
/// 
/// 	const MAX: i32 = 16 - 1;
/// 	let classic_iter = (0 .. 16_i32.pow(3))
/// 		.map(|i| position_function(i))
/// 		.filter(|pos|
/// 			/* Check 'bordered' condition */
/// 			pos.x() == 0 || pos.x() == MAX ||
/// 			pos.y() == 0 || pos.y() == MAX ||
/// 			pos.z() == 0 || pos.z() == MAX
/// 		);
/// 
/// 	/* Walk over both together */
/// 	for (b, w) in border.zip(classic_iter) {
/// 		assert_eq!(b, w)
/// 	}
/// }
/// ```
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

/// Full equivalent of 3-fold cycle.
/// 
/// # Example:
/// 
/// ```
/// use crate::app::utils::{
/// 	math::Int3,
/// 	terrain::chunk::iterator::SpaceIter,
/// };
/// 
/// fn test() {
/// 	let mut res1 = vec![];
/// 	let mut res2 = vec![];
/// 
/// 	/* [`SpaceIter`] equivalent */
/// 	for pos in SpaceIter::new(Int3::zero() .. Int3::all(16)) {
/// 		res1.push(pos)
/// 	}
/// 
/// 	/* Classic 3-fold cycle */
/// 	for x in 0..16 {
/// 	for y in 0..16 {
/// 	for z in 0..16 {
/// 		res2.push(Int3::new(x, y, z))
/// 	}}}
/// 
/// 	assert_eq!(res1, res2);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SpaceIter {
	curr: Int3,

	min: Int3,
	max: Int3,
}

impl SpaceIter {
	pub fn new(range: Range<Int3>) -> Self {
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

/// Walks around 3D array in very specific way.
/// It breaks standart 3-fold cycle into chunks
/// and walks in them like usual 3-fold cycle.
/// 
/// It is in some way relative to 3-fold cycle in 3-fold cycle...
/// 
/// # Example:
/// ```
/// use crate::app::utils::{
/// 	math::prelude::*,
/// 	terrain::chunk::iterator::{
/// 		ChunkSplitten, SpaceIter
/// 	},
/// };
/// 
/// fn example() {
/// 	let split = ChunkSplitten::new(Int3::all(16), Int3::all(2));
///		let space: Vec<_> = SpaceIter::new(Int3::zero() .. Int3::all(16)).collect();
///
///		for (entire, _) in split {
///			assert!(space.contains(&entire));
///		}
/// }
/// ```
struct ChunkSplitten {
	inner: SpaceIter,
	outer: SpaceIter,
	current: Option<Int3>,

	chunk_size: Int3,
}

impl ChunkSplitten {
	/// Creates new [`ChunkSplitten`] from [`Int3`] of sizes of
	/// entire data and [`Int3`] of chunk sizes in elements of entire data structure.
	/// 
	/// # Panic
	/// If entire chunk size is not divisible into smaller chunk size.
	pub fn new(entire: Int3, chunk_size: Int3) -> Self {
		/* Check that entire chunk are divisible into smaller ones */
		assert_eq!(entire % chunk_size, Int3::zero());

		let mut outer = SpaceIter::new(Int3::zero() .. entire / chunk_size);
		let current = outer.next().wunwrap();

		Self {
			inner: SpaceIter::new(Int3::zero() .. chunk_size),
			outer, current: Some(current), chunk_size,
		}
	}
}

impl Iterator for ChunkSplitten {
	// Types for outer and inner positions.
	type Item = (Int3, Int3);
	fn next(&mut self) -> Option<Self::Item> {
		let inner = self.inner.next().unwrap_or_else(|| {
			self.current = self.outer.next();
			self.inner = SpaceIter::new(Int3::zero() .. self.chunk_size);

			return self.inner.next().wunwrap()
		});
		
		let outer = self.current?;

		return Some((outer * self.chunk_size + inner, inner))
	}
}

#[cfg(test)]
mod space_iter_tests {
	use super::*;

	#[test]
	fn zero_start_simple() {
		let mut res1 = vec![];
		let mut res2 = vec![];

		for pos in SpaceIter::new(Int3::zero() .. Int3::all(5)) {
			res1.push(pos)
		}

		for x in 0..5 {
		for y in 0..5 {
		for z in 0..5 {
			res2.push(Int3::new(x, y, z))
		}}}

		assert_eq!(res1, res2);
	}

	#[test]
	fn zero_start_hard() {
		let mut res1 = vec![];
		let mut res2 = vec![];

		for pos in SpaceIter::new(Int3::zero() .. Int3::new(16, 4, 9)) {
			res1.push(pos)
		}

		for x in 0..16 {
		for y in 0..4 {
		for z in 0..9 {
			res2.push(Int3::new(x, y, z))
		}}}

		assert_eq!(res1, res2);
	}

	#[test]
	fn simple_start_simple() {
		let mut res1 = vec![];
		let mut res2 = vec![];

		for pos in SpaceIter::new(Int3::all(-5) .. Int3::all(5)) {
			res1.push(pos)
		}

		for x in -5..5 {
		for y in -5..5 {
		for z in -5..5 {
			res2.push(Int3::new(x, y, z))
		}}}

		assert_eq!(res1, res2);
	}

	#[test]
	fn hard_start_hard() {
		let mut res1 = vec![];
		let mut res2 = vec![];

		for pos in SpaceIter::new(Int3::new(-8, 2, -10) .. Int3::new(9, 5, -5)) {
			res1.push(pos)
		}

		for x in -8..9 {
		for y in 2..5 {
		for z in -10..-5 {
			res2.push(Int3::new(x, y, z))
		}}}

		assert_eq!(res1, res2);
	}

	#[test]
	fn uniqueness() {
		let iter = SpaceIter::new(Int3::new(-8, 2, -10) .. Int3::new(9, 5, -5));
		let mut map = std::collections::HashSet::new();

		for pos in iter {
			assert!(map.insert(pos));
		}
	}
}

#[cfg(test)]
mod border_test {
	use {
		crate::app::utils::terrain::chunk::{CHUNK_SIZE, CHUNK_VOLUME, position_function},
		super::*,
	};

	#[test]
	fn test1() {
		let border = CubeBorder::new(CHUNK_SIZE as i32);
		const MAX: i32 = CHUNK_SIZE as i32 - 1;

		for pos in border {
			eprintln!("{:?}", pos);

			assert!(
				pos.x() == 0 || pos.x() == MAX ||
				pos.y() == 0 || pos.y() == MAX ||
				pos.z() == 0 || pos.z() == MAX
			);
		}
	}

	#[test]
	fn test2() {
		let border = CubeBorder::new(CHUNK_SIZE as i32);
		const MAX: i32 = CHUNK_SIZE as i32 - 1;

		let works = (0..CHUNK_VOLUME)
			.map(|i| position_function(i))
			.filter(|pos|
				pos.x() == 0 || pos.x() == MAX ||
				pos.y() == 0 || pos.y() == MAX ||
				pos.z() == 0 || pos.z() == MAX
			);

		for (b, w) in border.zip(works) {
			assert_eq!(b, w)
		}
	}
}

#[cfg(test)]
mod splitten_tests {
	use super::*;

	#[test]
	fn space_contains_split() {
		let split = ChunkSplitten::new(Int3::all(16), Int3::all(2));
		let space: Vec<_> = SpaceIter::new(Int3::zero() .. Int3::all(16)).collect();

		for (entire, _) in split {
			assert!(space.contains(&entire));
		}
	}

	#[test]
	fn split_contains_space() {
		let (split, _): (Vec<_>, Vec<_>) = ChunkSplitten::new(Int3::all(16), Int3::all(2)).unzip();
		let space = SpaceIter::new(Int3::zero() .. Int3::all(16));

		for pos in space {
			assert!(split.contains(&pos));
		}
	}

	#[test]
	fn length() {
		let (entire, inner): (Vec<_>, Vec<_>) = ChunkSplitten::new(Int3::all(32), Int3::all(4)).unzip();
		let space: Vec<_> = SpaceIter::new(Int3::zero() .. Int3::all(32)).collect();

		assert_eq!(entire.len(), inner.len());
		assert_eq!(entire.len(), space.len());
	}

	#[test]
	fn print() {
		let split = ChunkSplitten::new(Int3::all(6), Int3::all(2));

		for (entire, inner) in split {
			eprintln!("{:?} in {:?}", inner, entire);
		}
	}

	#[test]
	fn uniqueness() {
		let split = ChunkSplitten::new(Int3::all(4), Int3::all(2));
		let mut set = std::collections::HashSet::new();

		for (entire, inner) in split {
			assert!(
				set.insert(entire),
				"Values are: {:?} in {:?}",
				inner, entire
			);
		}
	}
}