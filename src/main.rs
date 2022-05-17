#[macro_use]
extern crate glium;

use std::fs;

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

	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl");
	let program = glium::Program::from_source(&display, shaders.vertex_src.as_str(), shaders.fragment_src.as_str(), None).unwrap();

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

struct Shader {
	vertex_src: String,
	fragment_src: String
}

impl Shader {
	fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
		let vertex_shader_src = fs::read_to_string(vertex_shader_path).expect("Can't read vertex shader file!");
		let fragment_shader_src = fs::read_to_string(fragment_shader_path).expect("Can't read fragment shader file!");

		let shader = Shader { vertex_src: String::from(vertex_shader_src), fragment_src: String::from(fragment_shader_src) };

		return shader;
	}
}

implement_vertex!(Vertex, position, color);