use {
    crate::window::message_box::{MessageBox, MessageBoxError, MessageBoxSuccess},
    std::fmt::Display,
};

crate::module_constructor! {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panicked: {panic_info}");
        error_message("Program panicked", panic_info)
            .expect("failed to make error message");
    }))
}

/// Constructs error message box.
pub fn error_message(msg: &str, error: &dyn Display) -> Result<MessageBoxSuccess, MessageBoxError> {
    MessageBox::new("Error message:", &format!("{msg}: {error}"))
        .errored()
        .show()
}