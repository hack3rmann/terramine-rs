use crate::window::Window;
use crate::shader::Shader;
use crate::vertex_buffer::VertexBuffer;
use crate::glium;

static mut IS_GRAPHICS_INITIALIZED: bool = false;

pub struct Graphics {
	pub window: Window,
	pub vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
	pub primitive_type: Option<glium::index::NoIndices>,
	pub display: glium::Display,
	pub event_loop: Option<glium::glutin::event_loop::EventLoop<()>>,
	pub shaders: Option<Shader>
}

impl Graphics {
	pub fn initialize() -> Result<Self, &'static str> {
		unsafe {
			if IS_GRAPHICS_INITIALIZED {
				return Err("Attempting to initialize graphics twice! Graphics is already initialized!");
			} else {
				IS_GRAPHICS_INITIALIZED = true;
			}
		}

		let window = Window::from(1024, 768, false);
		let event_loop = glium::glutin::event_loop::EventLoop::new();
		let display = {
			let context_builder = glium::glutin::ContextBuilder::new();
			glium::Display::new(window.window_builder.clone(), context_builder, &event_loop).unwrap()
		};

		Ok (
			Graphics {
				window: window,
				vertex_buffer: None,
				primitive_type: None,
				display: display,
				event_loop: Some(event_loop),
				shaders: None
			}
		)
	}

	pub fn upload_vertex_buffer(&mut self, vertex_buffer: VertexBuffer) {
		self.vertex_buffer = Some(vertex_buffer.vertex_buffer);
		self.primitive_type = Some(vertex_buffer.indices);
	}

	pub fn upload_shaders(&mut self, shaders: Shader) {
		self.shaders = Some(shaders);
	}

	pub fn take_event_loop(&mut self) -> glium::glutin::event_loop::EventLoop<()> {
		if let None = self.event_loop {
			panic!("Graphics.event_loop haven't been initialized!")
		} else {
			let mut event_loop = None;
			std::mem::swap(&mut event_loop, &mut self.event_loop);
			event_loop.unwrap()
		}
	}

	pub fn take_shaders(&mut self) -> Shader {
		if let None = self.shaders {
			panic!("Graphics.shaders haven't beed initialized!")
		} else {
			let mut shaders = None;
			std::mem::swap(&mut shaders, &mut self.shaders);
			shaders.unwrap()
		}
	}

	pub fn take_vertex_buffer(&mut self) -> glium::VertexBuffer<Vertex> {
		if let None = self.vertex_buffer {
			panic!("Graphics.vertex_buffer haven't been initialized!")
		} else {
			let mut vertex_buffer = None;
			std::mem::swap(&mut vertex_buffer, &mut self.vertex_buffer);
			vertex_buffer.unwrap()
		}
	}

	pub fn take_privitive_type(&mut self) -> glium::index::NoIndices {
		if let None = self.primitive_type {
			panic!("Graphics.primitive_type haven't been initialized!")
		} else {
			let mut primitive_type = None;
			std::mem::swap(&mut primitive_type, &mut self.primitive_type);
			primitive_type.unwrap()
		}
	}
}

/* Vertex help struct */
#[derive(Copy, Clone)]
pub struct Vertex {
	pub position: [f32; 2],
	pub tex_coords: [f32; 2]
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, tex_coords);