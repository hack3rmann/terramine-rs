use {
    // FIXME(hack3rmann): user_io
    // crate::app::utils::{user_io::keyboard, cfg},
    imgui::Ui,
};

pub fn make_window<Label: AsRef<str>>(
    ui: &Ui,
    name: Label,
) -> imgui::Window<'_, '_, Label> {
    let mut result = ui.window(name);

    // FIXME(hack3rmann): user_io
    // if !keyboard::is_pressed(cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS) {
    //     result = result
    //         .movable(false)
    //         .collapsible(false)
    //         .resizable(false)
    // }

    result
}
