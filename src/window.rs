/**
 *  Adds container to window stuff
 */

/* Glium stuff */
use glium::glutin;
use std::fs;
use crate::user_io::{Keyboard, Mouse};

/* Window struct */
pub struct Window {
	pub window_builder: glium::glutin::window::WindowBuilder,
    pub width: i32,
    pub height: i32
}

pub enum Exit {
    None,
    Exit
}

impl Window {
	/// Constructs window.
	pub fn from(width: i32, height: i32, resizable: bool) -> Self {
		/* Loading icon from bitmap */
		let icon = {
			/* Bytes vector from bmp file */
			/* File formatted in BGRA */
			let mut raw_data = fs::read("src/image/TerramineIcon32p.bmp").unwrap();

			/* Bytemap pointer load from 4 bytes of file */
			/* This pointer is 4 bytes large and can be found on 10th byte position from file begin */
			let start_bytes: usize =	(raw_data[13] as usize) << 24 |
										(raw_data[12] as usize) << 16 |
										(raw_data[11] as usize) << 8  |
										(raw_data[10] as usize);

			/* Trim useless information */
			let raw_data = raw_data[start_bytes..].as_mut();

			/* Converting BGRA into RGBA formats */
			let mut current: usize = 0;
			while current <= raw_data.len() - 3 {
				raw_data.swap(current, current + 2);
				current += 4;
			}

			/* Upload data */
			let mut data = Vec::with_capacity(raw_data.len());
			data.extend_from_slice(raw_data);
			glutin::window::Icon::from_rgba(data, 32, 32).unwrap()
		};

		let window_builder = glutin::window::WindowBuilder::new()
			.with_decorations(true)
			.with_title("Terramine")
			.with_resizable(resizable)
            .with_inner_size(glutin::dpi::LogicalSize::new(width, height))
			.with_window_icon(Some(icon));

		Window {
            window_builder: window_builder,
            width: width,
            height: height
        }
	}

	/// Processing window messages.
	pub fn process_events(event: &glium::glutin::event::Event<()>, keyboard: &mut Keyboard, mouse: &mut Mouse) -> Exit {
		match event {
			/* Window events */
            glutin::event::Event::WindowEvent { event, .. } => match event {
				/* Close event */
                glutin::event::WindowEvent::CloseRequested => Exit::Exit,
				/* Keyboard input event */
				glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					/* Key matching */
					Some(key) => match key {
						_ => {
							/* If key is pressed then press it on virtual keyboard, if not then release it. */
							match input.state {
								glutin::event::ElementState::Pressed => {
									keyboard.press(key);
									Exit::None
								},
								glutin::event::ElementState::Released => {
									keyboard.release(key);
									Exit::None
								}
							}
						}
					}
					_ => Exit::None
				},
				/* Mouse buttons match. */
				glutin::event::WindowEvent::MouseInput { button, state, .. } => match state {
					/* If button is pressed then press it on virtual mouse, if not then release it. */
					glutin::event::ElementState::Pressed => {
						mouse.press(*button);
						Exit::None
					},
					glutin::event::ElementState::Released => {
						mouse.release(*button);
						Exit::None
					}
				},
				/* Cursor entered the window event. */
				glutin::event::WindowEvent::CursorEntered { .. } => {
					mouse.on_window = true;
					Exit::None
				},
				/* Cursor left the window. */
				glutin::event::WindowEvent::CursorLeft { .. } => {
					mouse.on_window = false;
					Exit::None
				},
				/* Cursor moved to new position. */
				glutin::event::WindowEvent::CursorMoved { position, .. } => {
					mouse.move_cursor(position.x as f32, position.y as f32);
					Exit::None
				}
                _ => Exit::None,
            },
			/* Glium events */
            glutin::event::Event::NewEvents(cause) => match cause {
				/* "Wait until" called event */
                glutin::event::StartCause::ResumeTimeReached { .. } => Exit::None,
				/* Window initialized event */
                glutin::event::StartCause::Init => Exit::None,
                _ => Exit::None,
            },
            _ => Exit::None,
        }
	}

	/// Window close function.
    pub fn exit(control_flow: &mut glutin::event_loop::ControlFlow) {
        *control_flow = glutin::event_loop::ControlFlow::Exit;
    }
}