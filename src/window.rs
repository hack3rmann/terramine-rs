/**
 *  Adds container to window stuff
 */

/* Glium stuff */
use glium::glutin;

/* Window struct */
pub struct Window {
	pub window_builder: glium::glutin::window::WindowBuilder
}

impl Window {
	/* Constructs window */
	pub fn from(resizable: bool) -> Self {
		let window_builder = glutin::window::WindowBuilder::new()
			.with_decorations(true)
			.with_title("Terramine")
			.with_resizable(resizable);

		let window = Window { window_builder: window_builder };

		return window;
	}
	/* Processing window messages */
	pub fn process_events(event: &glium::glutin::event::Event<()>, control_flow: &mut glium::glutin::event_loop::ControlFlow) {
		match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
	}
}