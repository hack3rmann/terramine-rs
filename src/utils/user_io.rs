/**
 * Keyboard IO handler
 */

pub use glium::glutin::event::{
	ElementState,
	VirtualKeyCode as KeyCode,
	MouseButton,
	Event,
	WindowEvent
};
use std::collections::HashMap;
use crate::utils::graphics::Graphics;

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

		return is_pressed;
	}
}

#[derive(Default)]
pub struct Mouse {
	pub inputs: HashMap<MouseButton, ElementState>,
	pub dx: f64,
	pub dy: f64,
	pub x: f64,
	pub y: f64,
	pub on_window: bool
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

	/// Moves virtual cursor to new position.
	pub fn move_cursor(&mut self, x: f64, y: f64) {
		self.dx += x - self.x;
		self.dy += y  - self.y;
		self.x = x;
		self.y = y;
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

	/// Update d/
	pub fn update(&mut self) {
		self.dx = 0.0;
		self.dy = 0.0;
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
	 			/* Cursor moved to new position. */
	 			WindowEvent::CursorMoved { position, .. } => {
					let position = position.to_logical(graphics.display.gl_window().window().scale_factor());
					let position = graphics.imguiw.scale_pos_from_winit(graphics.display.gl_window().window(), position);
	 				self.mouse.move_cursor(position.x, position.y);
	 			}
	            _ => (),
	        },
			_ => ()
		}
	}

	pub fn update(&mut self) {
		self.mouse.update();
	}
}