#![allow(dead_code)]

use {
    crate::prelude::*,
    std::time::Instant,
    parking_lot::Mutex,
};



pub use terramine_profiler_macros::profiler_target as profile;



pub mod prelude {
    pub use super::{
        profile,
        super::profiler,
        MeasureId,
    };
}



pub type MeasureId = u64;



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
    pub fn new(target_name: impl Into<String>) -> Self {
        Self {
            target_name: target_name.into(),
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
    pub id: MeasureId,
}

impl Measure {
    pub fn new(id: MeasureId) -> Self {
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
    pub profiles: HashMap<MeasureId, Profile>,
}



static IS_DRAWING_ENABLED: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler {
        profiles: HashMap::new(),
    });
}

/// Adds profile
pub fn add_profile(profile: Profile, id: MeasureId) {
    PROFILER.lock()
        .profiles
        .insert(id, profile);
}

/// Uploads measure
pub fn upload_measure(measure: &Measure) {
    PROFILER.lock()
        .profiles
        .get_mut(&measure.id)
        .unwrap_or_else(|| panic!("measure {measure:?} should be in measure map"))
        .measures
        .push(measure.value)
}

/// Starting capturing to to profile under given `id`.
pub fn start_capture(target_name: impl Into<String>, id: MeasureId) -> Measure {
    let is_already_captured = PROFILER.lock()
        .profiles
        .get(&id)
        .is_some();
    
    if !is_already_captured {
        add_profile(Profile::new(target_name), id)
    }

    Measure::new(id)
}

/// Updates profiler and builds ImGui window.
pub fn update_and_build_window(ui: &imgui::Ui, dt: TimeStep) {
    if keyboard::just_pressed(cfg::key_bindings::ENABLE_PROFILER_WINDOW) {
        let _ = IS_DRAWING_ENABLED.fetch_update(AcqRel, Relaxed, |prev| Some(!prev));
    }

    let mut lock = PROFILER.lock();
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
                frame_time: time_summary / dt.as_secs_f64(),
                time: time_summary,
                max_time: profile.max_time,
            }
        })
        .collect();
    
    build_window(ui, data);
    drop(lock);

    update();
}

/// Updates profiler:
/// * Clears measures
pub fn update() {
    let mut profiler = PROFILER.lock();
    for profile in profiler.profiles.values_mut() {
        profile.measures.clear()
    }
}

/// Builds ImGui window of capturing results
pub fn build_window(ui: &imgui::Ui, profiler_result: Vec<Data<'_>>) {
    use crate::app::utils::graphics::ui::imgui_ext::make_window;

    if !profiler_result.is_empty() && IS_DRAWING_ENABLED.load(Relaxed) {
        make_window(ui, "Profiler")
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



pub macro scope {
    ($name:expr) => {
        use $crate::profiler::{MeasureId, start_capture};
        const SCOPE_ID: MeasureId = ::const_random::const_random!(u64) as MeasureId;
        let _measure = start_capture($name, SCOPE_ID);
    },

    () => {
        $crate::profiler::scope!(
            const_format::formatcp!("scope #{}", ::const_random::const_random!(u64))
        );
    },
}
