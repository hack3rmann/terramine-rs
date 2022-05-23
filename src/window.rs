/**
 *  Adds container to window stuff
 */

/* Glium stuff */
use glium::glutin;

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
		let window_builder = glutin::window::WindowBuilder::new()
			.with_decorations(true)
			.with_title("Terramine")
			.with_resizable(resizable)
            .with_inner_size(glutin::dpi::LogicalSize::new(width, height));

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