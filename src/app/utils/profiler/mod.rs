#![allow(dead_code)]

pub extern crate profiler as profiler_target_macro;
pub use profiler_target_macro::profiler_target;

use {
    crate::app::utils::{
        time::timer::Timer,
        user_io::Keyboard,
        cfg,
    },
    std::{
        collections::HashMap,
        time::Instant,
        sync::Mutex,
    },
    lazy_static::lazy_static,
};

pub mod prelude {
    pub use super::{
        profiler_target as profile,
        super::profiler,
        Id,
    };
}

pub type Id = u64;
pub type DataSummary<'s> = Vec<Data<'s>>;

#[derive(Debug, Clone, Copy)]
pub struct Data<'s> {
    name: &'s str,
    call_freq: usize,
    frame_time: f64,
    time: f64,
    max_time: f64,
}

/// Represents profiler target.
#[derive(Debug)]
pub struct Profile {
    pub target_name: String,
    pub measures: Vec<f64>,
    pub max_time: f64,
}

impl Profile {
    /// Creates new profile
    pub fn new(target_name: &str) -> Self {
        Self {
            target_name: target_name.to_owned(),
            measures: vec![],
            max_time: 0.0
        }
    }
}

/// Represents a time measure with drop-stop.
#[derive(Debug)]
pub struct Measure {
    pub value: f64,
    pub now: Instant,
    pub id: Id,
}

impl Measure {
    pub fn new(id: Id) -> Self {
        Self { value: 0.0, now: Instant::now(), id }
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
    pub profiles: HashMap<Id, Profile>,
}

static DRAWING_ENABLED: Mutex<bool> = Mutex::new(false);

lazy_static! {
    static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler {
        profiles: HashMap::new(),
    });
}

/// Adds profile
pub fn add_profile(profile: Profile, id: Id) {
    PROFILER.lock()
        .expect("mutex should be not poisoned")
        .profiles
        .insert(id, profile);
}

/// Uploads measure
pub fn upload_measure(measure: &Measure) {
    PROFILER.lock()
        .expect("mutex should be not poisoned")
        .profiles
        .get_mut(&measure.id)
        .expect(&format!("measure {measure:?} should be in measure map"))
        .measures
        .push(measure.value)
}

/// Starting capturing to to profile under given `id`.
pub fn start_capture(target_name: &str, id: Id) -> Measure {
    let is_already_captured = PROFILER.lock()
        .expect("mutex should be not poisoned")
        .profiles
        .get(&id)
        .is_some();
    
    if !is_already_captured {
        add_profile(Profile::new(target_name), id)
    }

    Measure::new(id)
}

/// Updates profiler and builds ImGui window.
pub fn update_and_build_window(ui: &imgui::Ui, timer: &Timer, keyboard: &mut Keyboard) {
    if keyboard.just_pressed(cfg::key_bindings::ENABLE_PROFILER_WINDOW) {
        let mut guard = DRAWING_ENABLED.lock()
            .expect("DRAWING_ENABLED mutex shuold be not poisoned");
        *guard = !*guard;
    }

    let mut lock = PROFILER.lock()
        .expect("mutex should be not poisoned");
    let data = lock.profiles
        .iter_mut()
        .map(|(_, profile)| {
            let time_summary: f64 = profile.measures.iter()
                .copied()
                .sum();
            let cur_max = profile.measures.iter()
                .copied()
                .reduce(f64::max)
                .unwrap_or(0.0);
            profile.max_time = profile.max_time.max(cur_max);

            Data {
                name: profile.target_name.as_str(),
                call_freq: profile.measures.len(),
                frame_time: time_summary / timer.dt_as_f64(),
                time: time_summary,
                max_time: profile.max_time,
            }
        })
        .collect();
    
    build_window(ui, keyboard, data);
    drop(lock);

    update();
}

/// Updates profiler:
/// * Clears measures
pub fn update() {
    let mut lock = PROFILER.lock()
        .expect("mutex should be not poisoned");
    let profiles = lock.profiles
        .iter_mut();

    for (_, profile) in profiles {
        profile.measures.clear()
    }
}

/// Builds ImGui window of capturing results
pub fn build_window(ui: &imgui::Ui, keyboard: &Keyboard, profiler_result: DataSummary) {
    use crate::app::utils::graphics::ui::imgui_constructor::make_window;

    if profiler_result.len() != 0 && *DRAWING_ENABLED.lock().expect("mutex should be not poisoned") {
        make_window(ui, "Profiler", keyboard)
            .always_auto_resize(true)
            .build(|| {
            /* Build all elements. Separate only existing lines. */
            for (i, data) in profiler_result.iter().enumerate() {
                /* Target name */
                ui.text(data.name);

                /* Call count */
                ui.text(format!("Call per frame: {}", data.call_freq));

                /* Time that function need */
                ui.text(format!("Time: {:.3}ms", data.time * 1000.0));

                /* Percent of frame time */
                ui.text(format!("Frame time: {:.3}%", data.frame_time * 100.0));

                /* Percent of frame time */
                ui.text(format!("Max time: {:.3}ms", data.max_time * 1000.0));

                /* Separator to next result */
                if i != profiler_result.len() - 1 {
                    ui.separator();
                }
            }
        });
    }
}