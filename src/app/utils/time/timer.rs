#![allow(dead_code)]

use std::time::{Instant, Duration};

/// Provides easy time management
pub struct Timer {
	/* Time section */
	dt: f64,
	time: f32,
	last: Instant,

	/* FPS section */
	fps: f32,
	current_l: u32
}

impl Timer {
	/// Constructs new timer.
	pub fn new() -> Self {
		Timer { dt: 0.0, time: 0.0, last: Instant::now(), fps: 60.0, current_l: 0 }
	}

	/// Updates delta and full time.
	pub fn update(&mut self) {
		/* Now updates */
		let now = Instant::now();
		
		/* Update delta */
		self.dt = now.duration_since(self.last).as_secs_f64();

		/* Replace last frame with current */
		self.last = now;

		/* Add delta to time */
		self.time += self.dt as f32;

		if (self.current_l as f32) < self.time - 1.0 {
			self.current_l = self.time as u32;
			self.fps = 1.0 / self.dt as f32;
		}
	}

	/// Gives delta from last `update()` call
	pub fn dt_as_f32(&self) -> f32 { self.dt as f32 }

	/// Gives delta from last `update()` call
	pub fn dt_as_f64(&self) -> f64 { self.dt }

	/// Gives full time from last `update()` call
	pub fn time(&self) -> f32 { self.time }

	/// Gives fps from last `update()` call
	pub fn fps(&self) -> f32 { self.fps }

	/// Gives duration from last `update()` call
	pub fn duration(&self) -> Duration { Duration::from_secs_f64(self.dt) }
}