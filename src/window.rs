/**
 *  Adds container to window stuff
 */

/* Glium stuff */
use glium::glutin;
use std::fs;

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
			/* Bytes vector from file */
			let raw_data = fs::read("src/image/TerramineIcon32p.bmp").unwrap();

			/* Bytemap pointer load from 4 bytes of file */
			/* This pointer is 4 bytes large and can be found on 10th byte position from file begin */
			let start_bytes: usize =	(raw_data[13] as usize) << 24 |
										(raw_data[12] as usize) << 16 |
										(raw_data[11] as usize) << 8  |
										(raw_data[10] as usize);

			/* Trim useless information */
			let raw_data = &raw_data[start_bytes..];

			/* Upload data */
			let mut data = Vec::with_capacity(raw_data.len());
			data.extend_from_slice(raw_data);
			glutin::window::Icon::from_rgba(data, 32, 32).unwrap()
		};

		/* Build the window */
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
	pub fn process_events(event: &glium::glutin::event::Event<()>) -> Exit {
		match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    return Exit::Exit;
                },
				glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => match key {
						glutin::event::VirtualKeyCode::Escape => return Exit::Exit,
						_ => return Exit::None
					}
					_ => return Exit::None
				},
                _ => return Exit::None,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => Exit::None,
                glutin::event::StartCause::Init => Exit::None,
                _ => return Exit::None,
            },
            _ => return Exit::None,
        }
	}
	/// Window close function.
    pub fn exit(control_flow: &mut glutin::event_loop::ControlFlow) {
        *control_flow = glutin::event_loop::ControlFlow::Exit;
    }
}