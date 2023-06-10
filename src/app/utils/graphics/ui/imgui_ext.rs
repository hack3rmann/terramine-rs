use { crate::prelude::*, imgui::Ui };



pub type WindowBuilder = fn(&imgui::Ui);

pub static WINDOW_BUILDERS: Mutex<SmallVec<[WindowBuilder; 64]>> = const_default();

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
pub fn use_each_window_builder(ui: &imgui::Ui) {
    for build in WINDOW_BUILDERS.lock().iter() { build(ui) }
}



/// Makes an [`imgui`] window with moving, collapse and resize controls available
/// only if [`cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS`] key is pressed.
pub fn make_window<Label: AsRef<str>>(ui: &Ui, name: Label) -> imgui::Window<'_, '_, Label> {
    let mut result = ui.window(name);

    if !keyboard::is_pressed(cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS) {
        result = result
            .movable(false)
            .collapsible(false)
            .resizable(false)
    }

    result
}