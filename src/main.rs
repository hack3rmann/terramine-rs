#[macro_use]
extern crate glium;
extern crate image;
mod shader;
mod texture;
mod window;
mod graphics;
mod vertex_buffer;

/* Glium includes */
use glium::Surface;

/* Other files */
use shader::Shader;
use texture::Texture;
use window::Window;
use graphics::Graphics;
use vertex_buffer::VertexBuffer;

fn main() {
	/* Graphics initialization */
	let mut graphics = Graphics::initialize().unwrap();

	/* Texture loading */
	let texture = Texture::from("src/image/testSprite.png", &graphics.display).unwrap();

	/* Vertex buffer loading */
	let vertex_buffer = VertexBuffer::default(&graphics);
	vertex_buffer.bind(&mut graphics);

	/* Shader program */
	let shaders = Shader::new("src/vertex_shader.glsl", "src/fragment_shader.glsl", &graphics.display);
	graphics.upload_shaders(shaders);

	/* Temporary moves */
	let vertex_buffer = graphics.take_vertex_buffer();
	let indices = graphics.take_privitive_type();
	let shaders = graphics.take_shaders();
	
	/* Event loop run */
	graphics.take_event_loop().run(move |event, _, control_flow| {
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
