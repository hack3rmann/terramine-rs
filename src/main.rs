#[macro_use]
extern crate glium;

use glium::glutin;
use glium::Surface;

fn main() {
	let event_loop = glutin::event_loop::EventLoop::new();
	let wb = glutin::window::WindowBuilder::new()
		.with_title("Terramine")
		.with_resizable(false);
	let cb = glutin::ContextBuilder::new();
	let display = glium::Display::new(wb, cb, &event_loop).unwrap();

	let vertex1 = Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] };
	let vertex2 = Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] };
	let vertex3 = Vertex { position: [ 0.5, -0.25], color: [0.0, 0.0, 1.0] };
	let shape = vec![vertex1, vertex2, vertex3];

	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	let vertex_shader_src = r#"
		#version 140

		in vec2 position;
		in vec3 color;

		out vec3 u_Color;

		void main() {
			u_Color = color;

			gl_Position = vec4(position, 0.0, 1.0);
		}
	"#;
	let fragment_shader_src = r#"
		#version 140

		in vec3 u_Color;
		out vec4 color;

		void main() {
			color = vec4(u_Color, 1.0);
		}
	"#;

	let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

	event_loop.run(move |ev, _, control_flow| {
		let mut target = display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0);

		target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

		target.finish().unwrap();

		let next_frame_time = std::time::Instant::now() + std::time::Duration::from_micros(16_666_667);
		*control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

		match ev {
			glutin::event::Event::WindowEvent { event, .. } => match event {
				glutin::event::WindowEvent::CloseRequested => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
					return;
				},
				_ => return
			}, 
			_ => ()
		}
	});
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
	color: [f32; 3]
}

implement_vertex!(Vertex, position, color);