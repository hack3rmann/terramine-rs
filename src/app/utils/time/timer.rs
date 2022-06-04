#![allow(dead_code)]

use std::time::{Instant, Duration};

/// Provides easy time management
pub struct Timer {
	dt: f64,
	time: f32,
	last: Instant,
}

impl Timer {
	/// Constructs new timer.
	pub fn new() -> Self {
		Timer { dt: 0.0, time: 0.0, last: Instant::now() }
	}

	/// Updates delta and full time.
	pub fn update(&mut self) {
		let now = Instant::now();
		self.dt = now.duration_since(self.last).as_secs_f64();
		self.last = now;
		self.time += self.dt as f32;
	}

	/// Gives delta from last `update()` call
	pub fn dt_as_f32(&self) -> f32 { self.dt as f32 }

	/// Gives delta from last `update()` call
	pub fn dt_as_f64(&self) -> f64 { self.dt }

	/// Gives full time from last `update()` call
	pub fn time(&self) -> f32 { self.time }

	/// Gives fps from last `update()` call
	pub fn fps(&self) -> f32 { (1.0 / self.dt) as f32 }

	/// Gives duration from last `update()` call
	pub fn duration(&self) -> Duration { Duration::from_secs_f64(self.dt) }
}