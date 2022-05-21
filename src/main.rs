#[macro_use]
extern crate glium;
extern crate image;
mod shader;
mod texture;
mod window;

/* Glium includes */
use glium::glutin;
use glium::Surface;

/* Other files */
use shader::Shader;
use texture::Texture;
use window::Window;

fn main() {
	/* Graphics stuff */
	let event_loop = glutin::event_loop::EventLoop::new();
	let window = Window::from(false);
	let display = {
		let cb = glutin::ContextBuilder::new();
		glium::Display::new(window.window_builder, cb, &event_loop).unwrap()
	};

	/* Texture loading */
	let texture = Texture::from("src/image/testSprite.png", &display);

	/* Define vertices and triangle */
	let shape = vec! [
		Vertex { position: [-0.9, -0.15 ], tex_coords: [ 0.0, 0.0 ] },
		Vertex { position: [-0.9,  0.15 ], tex_coords: [ 0.0, 1.0 ] },
		Vertex { position: [ 0.9,  0.15 ], tex_coords: [ 1.0, 1.0 ] },
		Vertex { position: [-0.9, -0.15 ], tex_coords: [ 0.0, 0.0 ] },
		Vertex { position: [ 0.9,  0.15 ], tex_coords: [ 1.0, 1.0 ] },
		Vertex { position: [ 0.9, -0.15 ], tex_coords: [ 1.0, 0.0 ] }
	];

	/* Define vertex buffer */
	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl", &display);

	/* Event loop run */
	event_loop.run(move |event, _, control_flow| {
		/* Event match */
		Window::process_events(&event, control_flow);
		if *control_flow == glutin::event_loop::ControlFlow::Exit {
			return;
		}

		/* Control flow */
		/* FPS cutoff removed because it may lag */
		let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(/*16_666_667*/0);
		*control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: texture.with_mips()
		};

		/* Drawing process */
		let mut target = display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}

/* Vertex help struct */
#[derive(Copy, Clone)]
struct Vertex {
	position: [f32; 2],
	tex_coords: [f32; 2]
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, tex_coords);