#![allow(dead_code)]

use std::time::{Instant, Duration};

/// Provides easy time management.
#[derive(Debug)]
pub struct Timer {
    /* Time section */
    dt: f64,
    time: f32,
    last: Instant,

    /* FPS section */
    fps: f32,
    current_l: u32,
    measurments: u32,
    avg_fps: f32,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Constructs new timer.
    pub fn new() -> Self {
        Timer { dt: 0.0, time: 0.0, last: Instant::now(), fps: 60.0, current_l: 0, measurments: 1, avg_fps: 0.0 }
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

        /* Update avarage of fps */
        self.avg_fps = {
            let curr_measure = self.measurments as f32;
            let next_measure = (self.measurments + 1) as f32;
            let fps =  1.0 / self.dt_as_f32();
            self.avg_fps * (curr_measure / next_measure) + fps / next_measure
        };
        self.measurments += 1;

        /* Update stuff once a second */
        if (self.current_l as f32) < self.time - 1.0 {
            self.current_l = self.time as u32;
            self.fps = self.avg_fps;
            self.measurments = 1;
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