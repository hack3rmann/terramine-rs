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
	let image = image::load(
		Cursor::new(&include_bytes!("image/grass_side_norn.png")),
		image::ImageFormat::Png
	).unwrap().to_rgba8();
	let image_size = image.dimensions();
	let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_size);

	/* Graphics stuff */
	let event_loop = glutin::event_loop::EventLoop::new();
	let window = Window::from(640, 480, false);
	let cb = glutin::ContextBuilder::new();
	let display = glium::Display::new(window.window_builder, cb, &event_loop).unwrap();

	/* Define vertices and triangle */
	let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0 ] };
	let vertex2 = Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 1.0 ] };
	let vertex3 = Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0 ] };
	let vertex4 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0 ] };
	let vertex5 = Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0 ] };
	let vertex6 = Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0 ] };
	let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6];

	/* Define vertex buffer */
	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl");
	let program = glium::Program::from_source(&display, shaders.vertex_src.as_str(), shaders.fragment_src.as_str(), None).unwrap();

	let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

	/* Time stuff */
	let mut time: f32 = 0.0;
	let mut last_time = std::time::Instant::now();

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

		/* Update time stuff */
		time += (std::time::Instant::now() - last_time)
				.as_secs_f32() * 5.0;
		last_time = std::time::Instant::now();

		/* Uniforms set */
		let uniforms = uniform! {
			transform: [
				[ time.cos(), time.sin(), 0.0, 0.0],
				[-time.sin(), time.cos(), 0.0, 0.0],
				[ 0.0, 0.0, 1.0, 0.0],
				[ 0.0, 0.0, 0.0, 1.0f32]
			],
			time: time,
			tex: glium::uniforms::Sampler::new(&texture)
			.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
		};

		/* Drawing process */
		let mut target = display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
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