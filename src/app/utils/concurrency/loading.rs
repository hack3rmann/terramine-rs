#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loading {
	pub state: &'static str,
	pub percent: f64
}

impl Loading {
	/// Creates new `Loading` struct.
	#[allow(dead_code)]
    pub const fn new(state: &'static str, percent: f64) -> Self {
		Self { state, percent }
	}

	/// Gives `None` Loading.
	pub const fn none() -> Self {
		Self { state: "None", percent: 0.0 }
	}

	/// Constructs `Loading` from value and a range.
	pub fn from_range(state: &'static str, value: usize, range: std::ops::Range<usize>) -> Self {
		Self { state, percent: (value + 1) as f64 / (range.end - range.start) as f64 }
	}
}