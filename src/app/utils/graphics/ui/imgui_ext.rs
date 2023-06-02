use { crate::prelude::*, imgui::Ui };



pub type WindowBuilder = fn(&imgui::Ui);

thread_local! {
    pub static WINDOW_BUILDERS: RefCell<Vec<WindowBuilder>> = RefCell::new(Vec::with_capacity(32));
}

pub fn push_window_builder(builder: WindowBuilder) {
    WINDOW_BUILDERS.with_borrow_mut(|bulders| bulders.push(builder));
}

pub fn use_each_window_builder(ui: &imgui::Ui) {
    WINDOW_BUILDERS.with_borrow(|builders| {
        for build in builders.iter().copied() { build(ui) }
    })
}



pub fn make_window<Label: AsRef<str>>(
    ui: &Ui,
    name: Label,
) -> imgui::Window<'_, '_, Label> {
    let mut result = ui.window(name);

    if !keyboard::is_pressed(cfg::key_bindings::ENABLE_DRAG_AND_RESIZE_WINDOWS) {
        result = result
            .movable(false)
            .collapsible(false)
            .resizable(false)
    }

    result
}