#[macro_use]
extern crate glium;

/* Glium includes */
use glium::glutin;
use glium::Surface;

/* Other files */
mod shader;
use shader::Shader;

fn main() {
	/* Graphics stuff */
	let event_loop = glutin::event_loop::EventLoop::new();
	let wb = glutin::window::WindowBuilder::new()
		.with_decorations(true)
		.with_title("Terramine")
		.with_resizable(false);
	let cb = glutin::ContextBuilder::new();
	let display = glium::Display::new(wb, cb, &event_loop).unwrap();

	/* Define vertices and triangle */
	let vertex1 = Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] };
	let vertex2 = Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] };
	let vertex3 = Vertex { position: [ 0.5, -0.25], color: [0.0, 0.0, 1.0] };
	let shape = vec![vertex1, vertex2, vertex3];

	/* Define vertex buffer */
	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl");
	let program = glium::Program::from_source(&display, shaders.vertex_src.as_str(), shaders.fragment_src.as_str(), None).unwrap();

	/* Time stuff */
	let mut time: f32 = 0.0;
	let mut last_time = std::time::Instant::now();
	let mut dt = std::time::Instant::now() - last_time;

	/* Event loop run */
	event_loop.run(move |event, _, control_flow| {
		/* Event match */
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

		/* Control flow */
		let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

		/* Update time stuff */
		dt = std::time::Instant::now() - last_time;
		time += dt.as_secs_f32() * 10.0;
		last_time = std::time::Instant::now();

		/* Drawing process */
		let mut target = display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &program, &uniform! { time: time }, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}

/* Vertex help struct */
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
	color: [f32; 3]
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, color);