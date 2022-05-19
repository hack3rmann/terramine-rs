#[macro_use]
extern crate glium;
extern crate image;

/* STD stuff */
use std::io::Cursor;

/* Glium includes */
use glium::glutin;
use glium::Surface;

/* Other files */
mod shader;
mod window;
use shader::Shader;
use window::Window;

fn main() {
	/* Image loading */
	let image = image::load(
		Cursor::new(&include_bytes!("image/grass_side_norn.png")),
		image::ImageFormat::Png
	).unwrap().to_rgba8();
	let image_size = image.dimensions();
	let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_size);

	/* Graphics stuff */
	let event_loop = glutin::event_loop::EventLoop::new();
	let window = Window::from(false);
	let cb = glutin::ContextBuilder::new();
	let display = glium::Display::new(window.window_builder, cb, &event_loop).unwrap();

	/* Define vertices and triangle */
	let shape = vec! [
		Vertex { position: [-0.5, -0.5 ], tex_coords: [ 0.0, 0.0 ] },
		Vertex { position: [-0.5,  0.5 ], tex_coords: [ 0.0, 1.0 ] },
		Vertex { position: [ 0.5,  0.5 ], tex_coords: [ 1.0, 1.0 ] },
		Vertex { position: [-0.5, -0.5 ], tex_coords: [ 0.0, 0.0 ] },
		Vertex { position: [ 0.5,  0.5 ], tex_coords: [ 1.0, 1.0 ] },
		Vertex { position: [ 0.5, -0.5 ], tex_coords: [ 1.0, 0.0 ] }
	];

	/* Define vertex buffer */
	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl", &display);

	/* OpenGL texture 2d */
	let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

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
			/* Texture uniform with nearest filtering */
			tex: glium::uniforms::Sampler::new(&texture)
					.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
					.minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapNearest)
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