#[macro_use]
extern crate glium;
extern crate image;
mod shader;
mod texture;
mod window;
mod graphics;
mod vertex_buffer;
mod camera;

/* Glium includes */
use glium::Surface;

/* Other files */
use camera::Camera;
use shader::Shader;
use texture::Texture;
use window::Window;
use graphics::Graphics;
use vertex_buffer::VertexBuffer;

fn main() {
	/* Graphics initialization */
	let mut graphics = Graphics::initialize().unwrap();

	/* Camera handle */
	let mut camera = Camera::new();

	/* Texture loading */
	let texture = Texture::from("src/image/testSprite.png", &graphics.display).unwrap();

	/* Vertex buffer loading */
	let vertex_buffer = VertexBuffer::default(&graphics);
	vertex_buffer.bind(&mut graphics);

	/* Shader program */
	let shaders = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
	graphics.upload_shaders(shaders);

	/* Temporary moves */
	let vertex_buffer = graphics.take_vertex_buffer();
	let indices = graphics.take_privitive_type();
	let shaders = graphics.take_shaders();

	/* Time stuff */
	let time_start = std::time::Instant::now();
	let mut _time = time_start.elapsed().as_secs_f32();

	/* Event loop run */
	graphics.take_event_loop().run(move |event, _, control_flow| {
		/* Exit if window have that message */
		match Window::process_events(&event) {
			window::Exit::Exit => {
				Window::exit(control_flow);
				return;
			},
			_ => ()
		}

		/* Time refresh */
		_time = time_start.elapsed().as_secs_f32();

		/* Rotating camera */
		camera.set_rotation(1.0, _time * 2.0, _time, 0.0);

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: texture.with_mips(),
			time: _time,
			proj: camera.get_proj(),
			view: camera.get_view()
		};

		/* Drawing process */
		let mut target = graphics.display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}
