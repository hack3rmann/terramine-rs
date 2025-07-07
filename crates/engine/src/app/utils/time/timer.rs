#![allow(dead_code)]

use {
    crate::cfg::timer::N_FAMES_TO_MEASURE,
    std::time::{Duration, Instant},
};

/// Provides easy time management.
#[derive(Debug)]
pub struct Timer {
    pub dt: f32,

    pub time: f32,
    pub last_frame: Instant,

    pub fps: f32,
    pub frame_idx: usize,
    pub frames_sum: f32,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Constructs new timer.
    pub fn new() -> Self {
        Self {
            dt: 0.0,
            time: 0.0,
            last_frame: Instant::now(),
            fps: 0.0,
            frame_idx: 0,
            frames_sum: 0.0,
        }
    }

    /// Updates delta and full time.
    pub fn update(&mut self) {
        /* Now updates */
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_idx += 1;
        self.frames_sum += self.dt;

        if self.frame_idx >= N_FAMES_TO_MEASURE {
            self.frame_idx = 0;
            self.fps = N_FAMES_TO_MEASURE as f32 / self.frames_sum;
            self.frames_sum = 0.0;
        }
    }

    /// Gives duration from last `update()` call
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f32(self.dt)
    }
}
