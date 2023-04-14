use {
    crate::app::utils::window::message_box::{MessageBox, MessageBoxError, MessageBoxSuccess},
    std::fmt::Display,
};

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        error_message("Panic occured", panic_info)
            .expect("failed to make error message");
    }))
}

/// Constructs error message box.
pub fn error_message(msg: &str, error: &dyn Display) -> Result<MessageBoxSuccess, MessageBoxError> {
    MessageBox::new("Error message:", &format!("{msg}: {error}"))
        .errored()
        .show()
}