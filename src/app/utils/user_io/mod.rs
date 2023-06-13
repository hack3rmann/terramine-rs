//! 
//! IO handler
//! 

pub mod keyboard;
pub mod mouse;



use {
    crate::{prelude::*, window::Window},
    winit::event::{ElementState, Event, WindowEvent},
};



pub use winit::event::VirtualKeyCode as Key;



module_constructor! {
    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe { app::update::push_function_lock_free(keyboard::update) };
}



#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, IsVariant)]
pub enum KeyState {
    Pressed,
    Released,
}

impl ConstDefault for KeyState {
    const DEFAULT: Self = Self::Released;
}

impl Default for KeyState {
    fn default() -> Self { const_default() }
}

impl From<winit::event::ElementState> for KeyState {
    fn from(value: winit::event::ElementState) -> Self {
        use winit::event::ElementState::*;
        match value {
            Pressed => Self::Pressed,
            Released => Self::Released,
        }
    }
}

impl From<KeyState> for winit::event::ElementState {
    fn from(value: KeyState) -> Self {
        match value {
            KeyState::Pressed => Self::Pressed,
            KeyState::Released => Self::Released,
        }
    }
}



pub fn handle_event(event: &Event<()>, window: &Window) {
    macros::atomic_static! {
        static CURSOR_RECAPTURED: bool = false;
    }

    if let Event::WindowEvent { event, .. } = event {
        match event {
            // Close event
            WindowEvent::KeyboardInput { input, .. } => if let Some(key) = input.virtual_keycode {
                match input.state {
                    ElementState::Pressed => keyboard::press(key),
                    ElementState::Released => keyboard::release(key),
                }
            },

            // Mouse buttons match
            WindowEvent::MouseInput { button, state, .. } => match state {
                // If button is pressed then press it on virtual mouse, if not then release it
                ElementState::Pressed => mouse::press((*button).into()),
                ElementState::Released => mouse::release((*button).into()),
            },

            // Cursor entered the window event
            WindowEvent::CursorEntered { .. } =>
                mouse::IS_ON_WINDOW.store(true, Relaxed),

            // Cursor left the window
            WindowEvent::CursorLeft { .. } =>
                mouse::IS_ON_WINDOW.store(false, Relaxed),

            WindowEvent::Focused(focused) => {
                let mut is_recaptured = CURSOR_RECAPTURED.load(Acquire);
                
                let is_captured = mouse::is_captured();
                
                // If window has unfocused then release cursor
                if *focused && is_recaptured && !is_captured {
                    mouse::capture(window);
                    is_recaptured = false;
                } else if is_captured {
                    mouse::uncapture(window);
                    is_recaptured = true;
                }

                CURSOR_RECAPTURED.store(is_recaptured, Release);
            }
            _ => (),
        }
    }
}