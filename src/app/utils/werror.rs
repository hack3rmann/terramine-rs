use {
    crate::app::utils::window::message_box::{MessageBox, self},
    std::fmt::Debug,
};

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        error_message("Panic occured", &format!(
            "{msg}: {panic_info:?}",
            msg = panic_info
        )).expect("failed to make error message box");
    }))
}

/// Constructs error message box.
pub fn error_message(msg: &str, error: &dyn Debug) -> message_box::result::Result {
    MessageBox::new("Error message:", &format!("{msg}: {error:?}"))
        .errored()
        .show()
}