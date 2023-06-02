//! 
//! IO handler
//! 

use {
    crate::prelude::*,
    winapi::{
        um::winuser::GetCursorPos,
        shared::windef::POINT,
    },
    glium::glutin::{
        event::{
            ElementState,
            MouseButton,
            Event,
            WindowEvent
        },
        window::CursorGrabMode,
        dpi::PhysicalPosition,
    },
};

pub use glium::glutin::event::VirtualKeyCode as Key;

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
        INPUTS.write()
            .insert(key, ElementState::Pressed);
    }

    pub fn release(key: Key) {
        INPUTS.write().remove(&key);
    }

    pub fn is_pressed(key: Key) -> bool {
        let is_pressed = INPUTS.read()
            .contains_key(&key);
        let is_captured = IS_INPUT_CAPTURED.load(Relaxed);

        is_pressed && !is_captured
    }

    pub fn just_pressed(key: Key) -> bool {
        let inputs = INPUTS.read();

        let is_pressed = inputs.contains_key(&key);
        if is_pressed && !IS_INPUT_CAPTURED.load(Relaxed) {
            RELEASED_KEYS.lock()
                .insert(key);
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
            let mut released_keys = RELEASED_KEYS.lock();

            for key in keys {
                released_keys.insert(key);
            }
        }

        is_pressed && !is_captured
    }

    pub fn update() {
        let mut input = INPUTS.write();

        let mut released_keys = RELEASED_KEYS.lock();

        for key in released_keys.iter() {
            input.remove(key);
        }

        released_keys.clear();
    }
}

pub mod mouse {
    #![allow(dead_code)]

    use {
        super::*,
        portable_atomic::AtomicF32,
    };

    lazy_static! {
        pub(super) static ref INPUTS: RwLock<HashSet<MouseButton>> = RwLock::new(HashSet::new());
        pub(super) static ref RELEASED_KEYS: tokio::sync::Mutex<HashSet<MouseButton>> =
            tokio::sync::Mutex::new(HashSet::new());
    }

    pub(super) static DX: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static DY: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static X: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static Y: AtomicF32 = AtomicF32::new(0.0);
    pub(super) static IS_ON_WINDOW: AtomicBool = AtomicBool::new(false);
    pub(super) static IS_GRABBED: AtomicBool = AtomicBool::new(false);

    pub fn get_x() -> f32 { X.load(Relaxed) }
    pub fn get_y() -> f32 { Y.load(Relaxed) }
    pub fn get_dx_dt() -> f32 { DX.load(Relaxed) }
    pub fn get_dy_dt() -> f32 { DY.load(Relaxed) }

    pub fn press(button: MouseButton) {
        INPUTS.write().insert(button);
    }

    pub fn release(button: MouseButton) {
        INPUTS.write().remove(&button);
    }

    pub fn is_pressed(button: MouseButton) -> bool {
        INPUTS.read().contains(&button)
    }

    pub fn just_pressed(button: MouseButton) -> bool {
        let is_pressed = is_pressed(button);

        if is_pressed {
            tokio::task::block_in_place(|| RUNTIME.block_on(async {
                RELEASED_KEYS.lock()
                    .await
                    .insert(button);
            }));
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

    /// Update mouse delta.
    pub async fn update(window: &glium::glutin::window::Window) -> Result<(), MouseError> {
        {
            let mut released_keys = RELEASED_KEYS.lock().await;

            let mut inputs = INPUTS.write();

            for key in released_keys.iter() {
                inputs.remove(key);
            }

            released_keys.clear();
        }

        /* Get cursor position from WinAPI */
        let (x, y) = get_cursor_pos(window)?.as_tuple();
        let prev_x = X.load(Acquire);
        let prev_y = Y.load(Acquire);

        /* Update struct */
        X.store(x, Release);
        Y.store(y, Release);
        DX.store(x - prev_x, Release);
        DY.store(y - prev_y, Release);

        /* Get window size */
        let wsize = window.inner_size();

        /* If mouse is captured then not change mouse position and put cursor on center */
        if IS_GRABBED.load(Relaxed) {
            window.set_cursor_position(
                PhysicalPosition::new(wsize.width / 2, wsize.height / 2)
            ).expect("failed to set cursor position");
            
            X.store((wsize.width  / 2) as f32, Release);
            Y.store((wsize.height / 2) as f32, Release);
        }

        Ok(())
    }

    /// Gives cursor position in screen cordinates.
    pub fn get_cursor_screen_pos() -> Result<Int2, MouseError> {
        // Point cordinates struct
        let mut pt = POINT { x: 0, y: 0 };
        
        // Checks if WinAPI `GetCursorPos()` success then return cursor position else error
        let result = unsafe { GetCursorPos(&mut pt) };
        if result != 0 {
            Ok(veci!(pt.x, pt.y))
        } else {
            // `GetCursorPos()` returned `false` for some reason
            Err(MouseError::GetCursorPos(result))
        }
    }

    /// Gives cursor position in window cordinates.
    pub fn get_cursor_pos(window: &glium::glutin::window::Window) -> Result<vec2, MouseError> {
        let (x, y) = vec2::from(get_cursor_screen_pos()?).as_tuple();
        let window_pos = window.inner_position()?;

        Ok(vecf!(x - window_pos.x as f32, y - window_pos.y as f32))
    }

    /// Grabs the cursor for camera control.
    pub fn capture(window: &glium::glutin::window::Window) {
        window.set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .expect("failed to set cursor grab");
        window.set_cursor_visible(false);

        IS_GRABBED.store(true, Relaxed);
    }

    /// Releases cursor for standart input.
    pub fn uncapture(window: &glium::glutin::window::Window) {
        window.set_cursor_grab(CursorGrabMode::None)
            .expect("failed to release cursor");
        window.set_cursor_visible(true);

        IS_GRABBED.store(false, Relaxed);
    }

    /// Sets capture mode.
    pub fn set_capture(window: &glium::glutin::window::Window, is_captured: bool) {
        if is_captured {
            capture(window);
        } else {
            uncapture(window);
        }
    }

    #[derive(Debug, Error)]
    pub enum MouseError {
        #[error("failed to get cursor position, error code: {0}")]
        GetCursorPos(i32),

        #[error(transparent)]
        NotSupported(#[from] glium::glutin::error::NotSupportedError),
    }
}

pub fn handle_event(event: &Event<()>, window: &glium::glutin::window::Window) {
    static CURSOR_REGRABBED: Mutex<bool> = Mutex::new(false);

    if let Event::WindowEvent { event, .. } = event {
        match event {
            /* Close event */
            WindowEvent::KeyboardInput { input, .. } => if let Some(key) = input.virtual_keycode {
                match input.state {
                    ElementState::Pressed => keyboard::press(key),
                    ElementState::Released => keyboard::release(key),
                }
            },

            /* Mouse buttons match. */
            WindowEvent::MouseInput { button, state, .. } => match state {
                /* If button is pressed then press it on virtual mouse, if not then release it. */
                ElementState::Pressed =>
                    mouse::press(*button),

                ElementState::Released =>
                    mouse::release(*button),
            },

            /* Cursor entered the window event. */
            WindowEvent::CursorEntered { .. } =>
                mouse::IS_ON_WINDOW.store(true, Relaxed),

            /* Cursor left the window. */
            WindowEvent::CursorLeft { .. } =>
                mouse::IS_ON_WINDOW.store(false, Relaxed),

            WindowEvent::Focused(focused) => {
                /* If window has unfocused then release cursor. */
                let mut is_regrabbed = CURSOR_REGRABBED.lock();

                let is_grabbed = mouse::IS_GRABBED.load(Relaxed);
                if *focused && *is_regrabbed && !is_grabbed {
                    mouse::capture(window);
                    *is_regrabbed = false;
                } else if is_grabbed {
                    mouse::uncapture(window);
                    *is_regrabbed = true;
                }
            }
            _ => (),
        }
    }
}