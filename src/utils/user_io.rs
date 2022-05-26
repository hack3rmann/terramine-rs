/**
 * Keyboard IO handler
 */

pub use glium::glutin::event::{ElementState, VirtualKeyCode as KeyCode, MouseButton};
use std::collections::HashMap;

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
	pub dx: f32,
	pub dy: f32,
	pub x: f32,
	pub y: f32,
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
	pub fn move_cursor(&mut self, x: f32, y: f32) {
		self.dx = x - self.x;
		self.dy = y - self.y;
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
}