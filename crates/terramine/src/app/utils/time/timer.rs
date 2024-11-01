#![allow(dead_code)]

use {
    crate::prelude::*,
    cfg::timer::N_FAMES_TO_MEASURE,
    std::time::{Instant, Duration},
};



/// Provides easy time management.
#[derive(Debug)]
pub struct Timer {
    dt: TimeStep,

    time: Time,
    last_frame: Instant,

    fps: f32,
    frame_idx: usize,
    frames_sum: Duration,
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
            dt: const_default(),
            time: const_default(),
            last_frame: Instant::now(),
            fps: 0.0,
            frame_idx: 0,
            frames_sum: const_default(),
        }
    }

    /// Updates delta and full time.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame);
        self.last_frame = now;
        
        self.time += self.dt;

        self.frame_idx += 1;
        self.frames_sum += self.dt;

        // Measure fps once per `N_FAMES_TO_MEASURE` frames.
        if self.frame_idx >= N_FAMES_TO_MEASURE {
            self.frame_idx = 0;
            self.fps = N_FAMES_TO_MEASURE as f32 / self.frames_sum.as_secs_f32();
            self.frames_sum = default();
        }
    }

    /// Gives duration since `update()` call
    pub const fn time_step(&self) -> TimeStep { self.dt }

    /// Gives `fps` measured by [timer][Timer].
    pub const fn fps(&self) -> f32 { self.fps }

    /// Gives time since [timer][Timer] creation.
    pub const fn time(&self) -> Time { self.time }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "time: {:.1}s\ntime step: {:.1}ms\nfps: {:.1}",
            self.time.as_secs_f32(),
            1000.0 * self.dt.as_secs_f32(),
            self.fps
        )
    }
}



pub type TimeStep = Duration;
pub type Time = Duration;
