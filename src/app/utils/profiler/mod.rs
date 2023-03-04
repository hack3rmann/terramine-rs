#![allow(dead_code)]

pub extern crate profiler as profiler_target_macro;
pub use profiler_target_macro::profiler_target;

use {
    crate::app::utils::{
        time::timer::Timer,
        user_io::InputManager,
        cfg,
    },
    std::{
        collections::HashMap,
        time::Instant,
        sync::Mutex,
    },
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
    pub id: Id
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
    pub profiles: Option<HashMap<Id, Profile>>,
}

impl Profiler {
    /// Gives uninitialyzed version of `Profiler` to create static variable
    const fn uninitialized() -> Self { Profiler { profiles: None } }

    /// Initialyzes static
    pub fn initialize(&mut self) {
        self.profiles = Some(HashMap::new())
    }
}

static DRAWING_ENABLED: Mutex<bool> = Mutex::new(false);
static IS_INITIALIZED:  Mutex<bool> = Mutex::new(false);
static PROFILER: Mutex<Profiler> = Mutex::new(Profiler::uninitialized());

/// Initializes static profiler.
/// Can be called only once! If not then it will panic.
pub fn initialize() {
    let mut inited = IS_INITIALIZED.lock()
        .expect("mutex should be not poisoned");

    match *inited {
        false => {
            *inited = true;
            PROFILER.lock()
                .expect("mutex should be not poisoned")
                .initialize();
        },
        true => panic!("cannot initialize profiler twice!"),
    }
}

/// Adds profile
pub fn add_profile(profile: Profile, id: Id) {
    PROFILER.lock()
        .expect("mutex should be not poisoned")
        .profiles
        .as_mut()
        .expect("profiler should be initialized")
        .insert(id, profile);
}

/// Uploads measure
pub fn upload_measure(measure: &Measure) {
    PROFILER.lock()
        .expect("mutex should be not poisoned")
        .profiles
        .as_mut()
        .expect("profiler should be initialized")
        .get_mut(&measure.id)
        .expect(&format!("measure {measure:?} should be in measure map"))
        .measures
        .push(measure.value)
}

/// Starting capturing to to profile under given `id`.
pub fn start_capture(target_name: &str, id: Id) -> Measure {
    let mut lock = PROFILER.lock()
        .expect("mutex should be not poisoned");
    let new_profile_needed = lock.profiles
        .as_mut()
        .expect("profiler should be initialized")
        .get(&id)
        .is_none();
    drop(lock);
    
    if new_profile_needed {
        add_profile(Profile::new(target_name), id)
    }

    Measure::new(id)
}

/// Updates profiler and builds ImGui window.
pub fn update_and_build_window(ui: &imgui::Ui, timer: &Timer, input: &mut InputManager) {
    if input.keyboard.just_pressed(cfg::key_bindings::ENABLE_PROFILER_WINDOW) {
        let mut guard = DRAWING_ENABLED.lock()
            .expect("DRAWING_ENABLED mutex shuold be not poisoned");
        *guard = !*guard;
    }

    let mut lock = PROFILER.lock()
        .expect("mutex should be not poisoned");
    let data = lock.profiles
        .as_mut()
        .expect("profiler should be initialized")
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
    
    build_window(ui, input, data);
    drop(lock);

    update();
}

/// Updates profiler:
/// * Clears measures
pub fn update() {
    let mut lock = PROFILER.lock()
        .expect("mutex should be not poisoned");
    let profiles = lock.profiles
        .as_mut()
        .expect("profiler should be initialized")
        .iter_mut();

    for (_, profile) in profiles {
        profile.measures.clear()
    }
}

/// Builds ImGui window of capturing results
pub fn build_window(ui: &imgui::Ui, input: &InputManager, profiler_result: DataSummary) {
    if profiler_result.len() != 0 && *DRAWING_ENABLED.lock().expect("mutex should be not poisoned") {
        /* Create ImGui window */
        let mut window = ui.window("Profiler").always_auto_resize(true);

        /* Check if window can be moved or resized */
        if !input.keyboard.is_pressed(cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS) {
            window = window
                .resizable(false)
                .movable(false)
                .collapsible(false)
        }

        /* Ui building */
        window.build(|| {
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