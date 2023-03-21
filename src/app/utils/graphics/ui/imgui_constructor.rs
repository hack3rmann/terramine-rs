use {
    crate::app::utils::{user_io::Keyboard, cfg},
    imgui::Ui,
};

pub fn make_window<'ui, Label: AsRef<str>>(
    ui: &'ui Ui,
    name: Label,
    keyboard: &Keyboard
) -> imgui::Window<'ui, 'ui, Label> {
    let mut result = ui.window(name);

    if !keyboard.is_pressed(cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS) {
        result = result
            .movable(false)
            .collapsible(false)
            .resizable(false)
    }

    result
}