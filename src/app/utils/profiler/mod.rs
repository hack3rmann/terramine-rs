#![allow(dead_code)]

pub mod prelude {
    pub use super::{
        profiler_target as profile,
        super::profiler,
        ID,
    };
}

use {
    crate::app::utils::{
        werror::prelude::*,
        time::timer::Timer,
        user_io::{InputManager, KeyCode},
    },
    std::{
        collections::HashMap,
        time::Instant,
    },
};

pub extern crate profiler as profiler_target_macro;
pub use profiler_target_macro::profiler_target;

pub type ID = u64;
pub type DataSummary<'s> = Vec<(&'s String, usize, f64, f64)>;

/// Represents profiler target.
#[derive(Debug)]
pub struct Profile {
    pub target_name: String,
    pub measures: Vec<f64>,
}

impl Profile {
    /// Creates new profile
    pub fn new(target_name: &str) -> Self {
        Profile { target_name: target_name.to_owned(), measures: vec![] }
    }
}

/// Represents a time measure with drop-stop.
#[derive(Debug)]
pub struct Measure {
    pub value: f64,
    pub now: Instant,
    pub id: ID
}

impl Measure {
    pub fn new(id: ID) -> Self {
        Measure { value: 0.0, now: Instant::now(), id }
    }
}

impl Drop for Measure {
    fn drop(&mut self) {
        self.value = self.now.elapsed().as_secs_f64();
        upload_measure(self);
    }
}

/// Handles all profiles.
#[derive(Debug)]
pub struct Profiler {
    pub profiles: Option<HashMap<ID, Profile>>,
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
static mut IS_INITIALYZED: bool = false;

/// Initialyzes static
/// Can be called only once! If not then it should panic
pub fn initialyze() {
    unsafe {
        if !IS_INITIALYZED {
            IS_INITIALYZED = true;
            PROFILER.initialyze();
        } else {
            panic!("Can not initialyze profiler twice!");
        }
    }
}

/// Adds profile
pub fn add_profile(profile: Profile, id: ID) {
    unsafe {
        PROFILER.profiles.as_mut().wunwrap().insert(id, profile);
    }
}

/// Uploads measure
pub fn upload_measure(measure: &Measure) {
    unsafe {
        PROFILER.profiles.as_mut().wunwrap()
            .get_mut(&measure.id).wunwrap()
            .measures.push(measure.value);
    }
}

/// Starting capturing to to profile under given `id`
pub fn start_capture(target_name: &str, id: ID) -> Measure {
    unsafe {
        match PROFILER.profiles.as_mut().wunwrap().get(&id) {
            None => add_profile(Profile::new(target_name), id),
            _ => ()
        }
    };
    Measure::new(id)
}

/// Updates profiler and builds ImGui window.
pub fn update_and_build_window(ui: &imgui::Ui, timer: &Timer, input: &InputManager) {
    build_window(ui, input, get_result(timer));
    update();
}

/// Outputs a vector of results of function capturing:
/// * FunctionName
/// * NumOfCall
/// * FramePercent
/// * CallTime
pub fn get_result<'t, 's>(timer: &'t Timer) -> DataSummary<'s> {
    unsafe {
        PROFILER.profiles.as_ref().wunwrap()
            .iter()
            .map(|e| {
                let time_summary = e.1.measures.iter().sum::<f64>();
                (
                    &e.1.target_name,
                    e.1.measures.len(),
                    time_summary / timer.dt_as_f64(),
                    time_summary
                )
            })
            .collect()
    }
}

/// Updates profiler:
/// * Clears mesures
pub fn update() {
    unsafe {
        for profile in PROFILER.profiles.as_mut().wunwrap().iter_mut() {
            profile.1.measures.clear()
        }
    }
}

/// Builds ImGui window of capturing results
pub fn build_window(ui: &imgui::Ui, input: &InputManager, profiler_result: DataSummary) {
    if profiler_result.len() != 0 {
        /* Create ImGui window */
        let mut window = imgui::Window::new("Profiler").always_auto_resize(true);

        /* Check if window can be moved or resized */
        if !input.keyboard.is_pressed(KeyCode::I) {
            window = window
                .resizable(false)
                .movable(false)
                .collapsible(false)
        }

        /* Ui building */
        window.build(ui, || {
            /*
             * Build all elements. Speparate only existing lines
             */

            for (i, result) in profiler_result.iter().enumerate() {
                /* Target name */
                ui.text(result.0);

                /* Call count */
                ui.text(format!("Call per frame: {}", result.1));

                /* Time that function need */
                ui.text(format!("Time: {:.3}ms", result.3 * 1000.0));

                /* Percent of frame time */
                ui.text(format!("Frame time: {:.3}%", result.2 * 100.0));

                /* Separator to next result */
                if i != profiler_result.len() - 1 {
                    ui.separator();
                }
            }
        });
    }
}