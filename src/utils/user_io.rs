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
use super::graphics::Graphics;
use winapi::um::winuser::GetCursorPos;
use winapi::shared::windef::LPPOINT;
use winapi::shared::windef::POINT;

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

	/// Moves virtual cursor to new position.
	pub fn move_cursor(&mut self, x: f64, y: f64) {
		// if self.is_grabbed {
		// 	let cx: f64 = 1024.0 / 2.0;
		// 	let cy: f64 = 768.0 / 2.0;
		// 	if self.x >= cx && self.y >= cy {
		// 		if x > self.x || y > self.y {
		// 			self.dx = x - self.x;
		// 			self.dy = y - self.y;
		// 		}
		// 	} else if self.x < cx && self.y >= cy {
		// 		if x < self.x || y > self.y {
		// 			self.dx = x - self.x;
		// 			self.dy = y - self.y;
		// 		}
		// 	} else if self.x < cx && self.y < cy {
		// 		if x < self.x || y < self.y {
		// 			self.dx = x - self.x;
		// 			self.dy = y - self.y;
		// 		}
		// 	} else if self.x >= cx && self.y < cy {
		// 		if x > self.x || y < self.y {
		// 			self.dx = x - self.x;
		// 			self.dy = y - self.y;
		// 		}
		// 	}
		// } else {
		// 	self.dx += x - self.x;
		// 	self.dy += y - self.y;
		// }
		// self.x = x;
		// self.y = y;
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

	/// Update delta.
	pub fn update(&mut self, graphics: &Graphics) {
		let (x, y) = Self::get_cursor_pos(graphics).unwrap();
		self.dx = x - self.x;
		self.dy = y - self.y;
		self.x = x;
		self.y = y;

		let wsize = graphics.display.gl_window().window().inner_size();

		if self.is_grabbed {
			graphics.display.gl_window().window().set_cursor_position(
				glium::glutin::dpi::PhysicalPosition::new(wsize.width / 2, wsize.height / 2)
			).unwrap();
			self.x = (wsize.width / 2) as f64;
			self.y = (wsize.height / 2) as f64;
		}
	}

	/// Gives cursor position in screen cordinates.
	pub fn get_cursor_screen_pos() -> Result<(f64, f64), &'static str> {
		let pt: LPPOINT = &mut POINT { x: 0, y: 0 };
		/* Safe because unwraping pointers after checking result */
		if unsafe { GetCursorPos(pt) } != 0 {
			let x = unsafe { (*pt).x as f64 };
			let y = unsafe { (*pt).y as f64 };
			Ok((x, y))
		} else {
			Err("Can't get cursor position!")
		}
	}

	pub fn get_cursor_pos(graphics: &Graphics) -> Result<(f64, f64), &'static str> {
		let (x, y) = Self::get_cursor_screen_pos()?;
		let window_pos = graphics.display.gl_window().window().inner_position().unwrap();

		Ok((x - window_pos.x as f64, y - window_pos.y as f64))
	}

	/// Grabs the cursor for camera control.
	pub fn grab_cursor(&mut self, graphics: &Graphics) {
		graphics.display.gl_window().window().set_cursor_grab(true).unwrap();
		graphics.display.gl_window().window().set_cursor_visible(false);
		self.is_grabbed = true;
	}

	/// Releases cursor for standart input.
	pub fn release_cursor(&mut self, graphics: &Graphics) {
		graphics.display.gl_window().window().set_cursor_grab(false).unwrap();
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
					// let position = position.to_logical(graphics.display.gl_window().window().scale_factor());
					// let position = graphics.imguiw.scale_pos_from_winit(graphics.display.gl_window().window(), position);
	 				// self.mouse.move_cursor(position.x, position.y);
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