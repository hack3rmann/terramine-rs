#[macro_use]
extern crate glium;
extern crate image;
mod shader;
mod texture;
mod window;
mod graphics;

/* Glium includes */
use glium::glutin;
use glium::Surface;

/* Other files */
use shader::Shader;
use texture::Texture;
use window::Window;
use graphics::{Graphics, Vertex};

fn main() {
	/* Graphics stuff */
	let event_loop = glutin::event_loop::EventLoop::new();
	let graphics = Graphics::initialize(&event_loop).unwrap();

	/* Texture loading */
	let texture = Texture::from("src/image/testSprite.png", &graphics.display).unwrap();

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
	let vertex_buffer = glium::VertexBuffer::new(&graphics.display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl", &graphics.display);

	/* Event loop run */
	event_loop.run(move |event, _, control_flow| {
		/* Exit if window have that message */
		if let window::Exit::Exit = Window::process_events(&event) {
			Window::exit(control_flow);
			return;
		}

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: texture.with_mips()
		};

		/* Drawing process */
		let mut target = graphics.display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}
