/*
 * Keyboard IO handler
 */

pub use glium::glutin::event::{
    ElementState,
    VirtualKeyCode as KeyCode,
    MouseButton,
    Event,
    WindowEvent
};

use {
    crate::app::utils::werror::prelude::*,
    super::graphics::Graphics,
    winapi::{
        um::winuser::GetCursorPos,
        shared::windef::{LPPOINT, POINT},
    },
    std::collections::HashMap,
};

/// Keyboard handler.
#[derive(Default)]
pub struct Keyboard {
    pub inputs: HashMap<KeyCode, ElementState>
}

impl Keyboard {
    /// Constructs keyboard with no keys are pressed.
    #[allow(dead_code)]
    pub fn new() -> Self { Default::default() }

    /// Presses key on virtual keyboard.
    pub fn press(&mut self, key: KeyCode) {
        self.inputs.insert(key, ElementState::Pressed);
    }

    /// Releases key on virtual keyboard.
    pub fn release(&mut self, key: KeyCode) {
        self.inputs.remove(&key);
    }

    /// Checks virtual key is pressed.
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.inputs.contains_key(&key)
    }

    /// Checks virtual is pressed then release it.
    pub fn just_pressed(&mut self, key: KeyCode) -> bool {
        let is_pressed = self.inputs.contains_key(&key);
        self.release(key);

        return is_pressed
    }

    /// Checks combination is pressed.
    #[allow(dead_code)]
    pub fn is_pressed_combo(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|key| self.inputs.contains_key(key))
    }

    /// Checks combination just pressed.
    #[allow(dead_code)]
    pub fn just_pressed_combo(&mut self, keys: &[KeyCode]) -> bool {
        if keys.iter().all(|key| self.inputs.contains_key(key)) {
            keys.iter().for_each(|&key| self.release(key));
            return true
        } else { false }
    }
}

#[derive(Default)]
pub struct Mouse {
    pub inputs: HashMap<MouseButton, ElementState>,
    pub dx: f64,
    pub dy: f64,
    pub x: f64,
    pub y: f64,
    pub on_window: bool,
    pub is_grabbed: bool,
}

#[allow(dead_code)]
impl Mouse {
    /// Constructs Mouse with no buttons are pressed.
    pub fn new() -> Self { Default::default() }

    /// Presses virtual mouse button.
    pub fn press(&mut self, button: MouseButton) {
        self.inputs.insert(button, ElementState::Pressed);
    }

    /// Releases virtual mouse button.
    pub fn release(&mut self, button: MouseButton) {
        self.inputs.remove(&button);
    }

    /// Checks if left mouse button pressed.
    pub fn is_left_pressed(&self) -> bool {
        self.inputs.contains_key(&MouseButton::Left)
    }

    /// Cheks if right mouse button pressed.
    pub fn is_right_pressed(&self) -> bool {
        self.inputs.contains_key(&MouseButton::Right)
    }

    /// Checks if middle mouse button pressed.
    pub fn is_middle_pressed(&self) -> bool {
        self.inputs.contains_key(&MouseButton::Middle)
    }

    /// Checks if left mouse button pressed then releases it.
    pub fn just_left_pressed(&mut self) -> bool {
        let pressed = self.inputs.contains_key(&MouseButton::Left);
        self.release(MouseButton::Left);
        return pressed;
    }

    /// Cheks if right mouse button pressed.
    pub fn just_right_pressed(&mut self) -> bool {
        let pressed = self.inputs.contains_key(&MouseButton::Right);
        self.release(MouseButton::Right);
        return pressed;
    }

    /// Checks if middle mouse button pressed.
    pub fn just_middle_pressed(&mut self) -> bool {
        let pressed = self.inputs.contains_key(&MouseButton::Middle);
        self.release(MouseButton::Middle);
        return pressed;
    }

    /// Update mouse delta.
    pub fn update(&mut self, graphics: &Graphics) {
        /* Get cursor position from WinAPI */
        let (x, y) = Self::get_cursor_pos(graphics).wunwrap();

        /* Update struct */
        self.dx = x - self.x;
        self.dy = y - self.y;
        self.x = x;
        self.y = y;

        /* Get window size */
        let wsize = graphics.display.gl_window().window().inner_size();

        /* If cursor grabbed then not change mouse position and put cursor on center */
        if self.is_grabbed {
            graphics.display.gl_window().window().set_cursor_position(
                glium::glutin::dpi::PhysicalPosition::new(wsize.width / 2, wsize.height / 2)
            ).wunwrap();
            
            self.x = (wsize.width  / 2) as f64;
            self.y = (wsize.height / 2) as f64;
        }
    }

    /// Gives cursor position in screen cordinates.
    pub fn get_cursor_screen_pos() -> Result<(f64, f64), &'static str> {
        /* Point cordinates struct */
        let pt: LPPOINT = &mut POINT { x: 0, y: 0 };
        
        /* Checks if WinAPI `GetCursorPos()` success then return cursor position else error */
        if unsafe { GetCursorPos(pt) } != 0 {
            /* Safe because unwraping pointers after checking result */
            let x = unsafe { (*pt).x as f64 };
            let y = unsafe { (*pt).y as f64 };
            Ok((x, y))
        }
        
        else {
            /* `GetCursorPos()` returned `false` for some reason */
            Err("Can't get cursor position!")
        }
    }

    /// Gives cursor position in window cordinates.
    pub fn get_cursor_pos(graphics: &Graphics) -> Result<(f64, f64), &'static str> {
        let (x, y) = Self::get_cursor_screen_pos()?;
        let window_pos = graphics.display.gl_window().window().inner_position().wunwrap();

        Ok((x - window_pos.x as f64, y - window_pos.y as f64))
    }

    /// Grabs the cursor for camera control.
    pub fn grab_cursor(&mut self, graphics: &Graphics) {
        graphics.display.gl_window().window().set_cursor_grab(true).wunwrap();
        graphics.display.gl_window().window().set_cursor_visible(false);
        self.is_grabbed = true;
    }

    /// Releases cursor for standart input.
    pub fn release_cursor(&mut self, graphics: &Graphics) {
        graphics.display.gl_window().window().set_cursor_grab(false).wunwrap();
        graphics.display.gl_window().window().set_cursor_visible(true);
        self.is_grabbed = false;
    }
}

/// Contains both input types: `keyboard` and `mouse`.
#[derive(Default)]
pub struct InputManager {
    pub keyboard: Keyboard,
    pub mouse: Mouse
}

impl InputManager {
    /// Constructs manager with default values.
    pub fn new() -> Self { Default::default() }

    pub fn handle_event(&mut self, event: &Event<()>, graphics: &Graphics) {
        static mut CURSOR_REGRABBED: bool = false;
        
        match event {
            /* Window events */
            Event::WindowEvent { event, .. } => match event {
                 /* Close event */
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                     /* Key matching */
                     Some(key) => match key {
                         _ => {
                             /* If key is pressed then press it on virtual keyboard, if not then release it. */
                             match input.state {
                                 ElementState::Pressed => {
                                     self.keyboard.press(key);
                                 },
                                 ElementState::Released => {
                                     self.keyboard.release(key);
                                 }
                             }
                         }
                     }
                     _ => ()
                 },

                 /* Mouse buttons match. */
                 WindowEvent::MouseInput { button, state, .. } => match state {
                     /* If button is pressed then press it on virtual mouse, if not then release it. */
                     ElementState::Pressed => {
                         self.mouse.press(*button);
                     },

                     ElementState::Released => {
                         self.mouse.release(*button);
                     }
                 },

                 /* Cursor entered the window event. */
                 WindowEvent::CursorEntered { .. } => {
                     self.mouse.on_window = true;
                 },

                 /* Cursor left the window. */
                 WindowEvent::CursorLeft { .. } => {
                     self.mouse.on_window = false;
                 },

                WindowEvent::Focused(focused) => {
                    /* If window has unfocused then release cursor. */
                    if *focused {
                        if unsafe { CURSOR_REGRABBED } {
                            if !self.mouse.is_grabbed {
                                self.mouse.grab_cursor(graphics);
                                unsafe { CURSOR_REGRABBED = false };
                            }
                        }
                    }
                    
                    else {
                        if self.mouse.is_grabbed {
                            self.mouse.release_cursor(graphics);
                            unsafe { CURSOR_REGRABBED = true };
                        }
                    }
                }
                _ => (),
            },
            _ => ()
        }
    }

    pub fn update(&mut self, graphics: &Graphics) {
        self.mouse.update(graphics);
    }
}