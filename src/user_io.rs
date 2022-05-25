/**
 * Keyboard IO handler
 */

pub use glium::glutin::event::{ElementState, VirtualKeyCode as KeyCode};
use std::collections::HashMap;

/// Keyboard handler.
#[derive(Default)]
pub struct Keyboard {
	pub inputs: HashMap<KeyCode, ElementState>
}

impl Keyboard {
	/// Constructs keyboard with no keys are pressed.
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