use std::collections::HashMap;

/// Represents profiler target
pub struct Profile {
	target_name: String,
	measures: Vec<f64>,
}

/// Handles all profiles
pub struct Profiler {
	profiles: Option<HashMap<u64, Profile>>,
}

impl Profiler {
	/// Gives uninitialyzed version of `Profiler` to create static variable
	const fn uninitialized() -> Self { Profiler { profiles: None } }

	/// Initialyzes static
	pub fn initialyze(&mut self) {
		self.profiles = Some(HashMap::new())
	}
}

pub static mut PROFILER: Profiler = Profiler::uninitialized();