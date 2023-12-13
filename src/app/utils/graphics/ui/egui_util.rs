use crate::prelude::*;



pub type WindowBuilder = fn(&mut egui::Context);

pub static WINDOW_BUILDERS: Mutex<SmallVec<[WindowBuilder; 64]>>
    = const_default();

/// Adds a function to window builders list without aquireing [`Mutex`]'s lock.
pub fn push_window_builder(builder: WindowBuilder) {
    WINDOW_BUILDERS.lock().push(builder)
}

/// Adds a function to window builders list without aquireing [`Mutex`]'s lock.
/// 
/// # Safety
/// 
/// - should be called on main thread.
/// - there's no threads pushing update functions.
pub unsafe fn push_window_builder_lock_free(builder: WindowBuilder) {
    WINDOW_BUILDERS
        .data_ptr()
        .as_mut()
        .unwrap_unchecked()
        .push(builder);
}

/// Applies builders to `ui`.
pub fn use_each_window_builder(ctx: &mut egui::Context) {
    for build in WINDOW_BUILDERS.lock().iter() {
        build(ctx)
    }
}



pub trait ShowDebugUi {
    fn show_debug_ui(&mut self, ctx: &mut egui::Context);
}
assert_obj_safe!(ShowDebugUi);