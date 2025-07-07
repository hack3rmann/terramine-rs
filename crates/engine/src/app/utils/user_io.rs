//!
//! IO handler
//!

use {
    crate::prelude::*,
    std::sync::{Mutex, RwLock},
};

use glium::winit::dpi::PhysicalPosition;
use glium::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use glium::winit::window::{CursorGrabMode, Window};
use winit::keyboard::PhysicalKey;

pub use glium::winit::keyboard::KeyCode as Key;

#[cfg(windows)]
use winapi::{shared::windef::POINT, um::winuser::GetCursorPos};

pub mod keyboard {
    #![allow(dead_code)]

    use super::*;

    lazy_static! {
        pub static ref INPUTS: RwLock<HashMap<Key, ElementState>> = RwLock::new(HashMap::new());
        pub static ref RELEASED_KEYS: Mutex<HashSet<Key>> = Mutex::new(HashSet::new());
    }

    pub static IS_INPUT_CAPTURED: AtomicBool = AtomicBool::new(false);

    pub fn set_input_capture(capture: bool) {
        IS_INPUT_CAPTURED.store(capture, Relaxed);
    }

    pub fn press(key: Key) {
        INPUTS.write().unwrap().insert(key, ElementState::Pressed);
    }

    pub fn release(key: Key) {
        INPUTS.write().unwrap().remove(&key);
    }

    pub fn is_pressed(key: Key) -> bool {
        let is_pressed = INPUTS.read().unwrap().contains_key(&key);
        let is_captured = IS_INPUT_CAPTURED.load(Relaxed);

        is_pressed && !is_captured
    }

    pub fn just_pressed(key: Key) -> bool {
        let inputs = INPUTS.read().unwrap();

        let is_pressed = inputs.contains_key(&key);
        if is_pressed && !IS_INPUT_CAPTURED.load(Relaxed) {
            RELEASED_KEYS.lock().unwrap().insert(key);
            true
        } else {
            false
        }
    }

    pub fn is_pressed_combo(keys: impl IntoIterator<Item = Key>) -> bool {
        keys.into_iter().all(is_pressed)
    }

    pub fn just_pressed_combo<Iter>(keys: Iter) -> bool
    where
        Iter: IntoIterator<Item = Key>,
        Iter::IntoIter: Clone,
    {
        let keys = keys.into_iter();
        let is_pressed = is_pressed_combo(keys.clone());
        let is_captured = IS_INPUT_CAPTURED.load(Relaxed);

        if is_pressed && !is_captured {
            let mut released_keys = RELEASED_KEYS.lock().unwrap();

            for key in keys {
                released_keys.insert(key);
            }
        }

        is_pressed && !is_captured
    }

    pub fn update() {
        let mut input = INPUTS.write().unwrap();

        let mut released_keys = RELEASED_KEYS.lock().unwrap();

        for key in released_keys.iter() {
            input.remove(key);
        }

        released_keys.clear();
    }
}

pub mod mouse {
    #![allow(dead_code)]

    use {super::*, portable_atomic::AtomicF32};

    lazy_static! {
        pub(super) static ref INPUTS: RwLock<HashSet<MouseButton>> = RwLock::new(HashSet::new());
        pub(super) static ref RELEASED_KEYS: Mutex<HashSet<MouseButton>> =
            Mutex::new(HashSet::new());
    }

    pub(super) static DX: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static DY: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static X: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static Y: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static IS_ON_WINDOW: AtomicBool = AtomicBool::new(false);
    pub(super) static IS_GRABBED: AtomicBool = AtomicBool::new(false);

    pub fn get_x() -> f32 {
        X.load(Relaxed)
    }
    pub fn get_y() -> f32 {
        Y.load(Relaxed)
    }
    pub fn get_dx_dt() -> f32 {
        DX.load(Relaxed)
    }
    pub fn get_dy_dt() -> f32 {
        DY.load(Relaxed)
    }

    pub fn press(button: MouseButton) {
        INPUTS.write().unwrap().insert(button);
    }

    pub fn release(button: MouseButton) {
        INPUTS.write().unwrap().remove(&button);
    }

    pub fn is_pressed(button: MouseButton) -> bool {
        INPUTS.read().unwrap().contains(&button)
    }

    pub fn just_pressed(button: MouseButton) -> bool {
        let is_pressed = is_pressed(button);

        if is_pressed {
            RELEASED_KEYS.lock().unwrap().insert(button);
        }

        is_pressed
    }

    pub fn is_left_pressed() -> bool {
        is_pressed(MouseButton::Left)
    }

    pub fn is_right_pressed() -> bool {
        is_pressed(MouseButton::Right)
    }

    pub fn is_middle_pressed() -> bool {
        is_pressed(MouseButton::Middle)
    }

    pub fn just_left_pressed() -> bool {
        just_pressed(MouseButton::Left)
    }

    pub fn just_right_pressed() -> bool {
        just_pressed(MouseButton::Right)
    }

    pub fn just_middle_pressed() -> bool {
        just_pressed(MouseButton::Middle)
    }

    pub fn handle_event(event: &Event<()>, _window: &Window) {
        if let Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } = event
        {
            let prev_x = X.load(Acquire);
            let prev_y = Y.load(Acquire);
            let prev_dx = DX.load(Acquire);
            let prev_dy = DY.load(Acquire);
            let dx = prev_dx + position.x as f32 - prev_x;
            let dy = prev_dy + position.y as f32 - prev_y;

            X.store(position.x as f32, Release);
            Y.store(position.y as f32, Release);
            DX.store(dx, Release);
            DY.store(dy, Release);
        }
    }

    /// Update mouse delta.
    pub fn update(window: &Window) -> Result<(), MouseError> {
        {
            let mut released_keys = RELEASED_KEYS.lock().unwrap();

            let mut inputs = INPUTS.write().expect("rwlock should be not poisoned");

            for key in released_keys.iter() {
                inputs.remove(key);
            }

            released_keys.clear();
        }

        let window_size = window.inner_size();
        let _centre = PhysicalPosition::new(window_size.width / 2, window_size.height / 2);

        DX.store(0.0, Release);
        DY.store(0.0, Release);

        /* If cursor grabbed then not change mouse position and put cursor on center */
        if IS_GRABBED.load(Relaxed) {
            // X.store(0.5 * window_size.width as f32, Release);
            // Y.store(0.5 * window_size.height as f32, Release);

            // window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
            // window.set_cursor_position(centre).unwrap();
            // window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        }

        Ok(())
    }

    /// Grabs the cursor for camera control.
    pub fn grab_cursor(window: &Window) {
        window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        window.set_cursor_visible(true);

        IS_GRABBED.store(true, Relaxed);
    }

    /// Releases cursor for standart input.
    pub fn release_cursor(window: &Window) {
        window.set_cursor_grab(CursorGrabMode::None).unwrap();
        window.set_cursor_visible(true);

        IS_GRABBED.store(false, Relaxed);
    }

    #[derive(Debug, Error)]
    pub enum MouseError {
        #[error("failed to get cursor position, error code: {0}")]
        GetCursorPos(i32),

        #[error("not supported: {0}")]
        NotSupported(#[from] glium::winit::error::NotSupportedError),
    }
}

pub fn handle_event(event: &Event<()>, window: &Window) {
    static CURSOR_REGRABBED: Mutex<bool> = Mutex::new(false);

    mouse::handle_event(event, window);

    if let Event::WindowEvent { event, .. } = event {
        match event {
            /* Close event */
            WindowEvent::KeyboardInput { event, .. } => match (event.state, event.physical_key) {
                (ElementState::Pressed, PhysicalKey::Code(code)) => keyboard::press(code),
                (ElementState::Released, PhysicalKey::Code(code)) => keyboard::release(code),
                _ => (),
            },

            /* Mouse buttons match. */
            WindowEvent::MouseInput { button, state, .. } => match state {
                /* If button is pressed then press it on virtual mouse, if not then release it. */
                ElementState::Pressed => mouse::press(*button),

                ElementState::Released => mouse::release(*button),
            },

            /* Cursor entered the window event. */
            WindowEvent::CursorEntered { .. } => mouse::IS_ON_WINDOW.store(true, Relaxed),

            /* Cursor left the window. */
            WindowEvent::CursorLeft { .. } => mouse::IS_ON_WINDOW.store(false, Relaxed),

            WindowEvent::Focused(focused) => {
                /* If window has unfocused then release cursor. */
                let mut is_regrabbed = CURSOR_REGRABBED.lock().unwrap();

                let is_grabbed = mouse::IS_GRABBED.load(Relaxed);
                if *focused && *is_regrabbed && !is_grabbed {
                    mouse::grab_cursor(window);
                    *is_regrabbed = false;
                } else if is_grabbed {
                    mouse::release_cursor(window);
                    *is_regrabbed = true;
                }
            }
            _ => (),
        }
    }
}
