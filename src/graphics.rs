use crate::window::{self, Window};
use crate::glium;
use glium::Surface;

static mut IS_GRAPHICS_INITIALIZED: bool = false;

pub struct Graphics {
	pub window: Window,
	pub vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
	pub display: glium::Display
}

impl Graphics {
	pub fn initialize(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> Result<Self, &'static str> {
		unsafe {
			if IS_GRAPHICS_INITIALIZED {
				return Err("Attempting to initialize graphics twice! Graphics is already initialized!");
			} else {
				IS_GRAPHICS_INITIALIZED = true;
			}
		}

		let window = Window::from(1024, 768, false);
		let display = {
			let context_builder = glium::glutin::ContextBuilder::new();
			glium::Display::new(window.window_builder.clone(), context_builder, &event_loop).unwrap()
		};

		Ok (
			Graphics {
				window: window,
				vertex_buffer: None,
				display: display
			}
		)
	}

	pub fn upload_vertex_buffer(&mut self, vertex_buffer: glium::VertexBuffer<Vertex>) {
		self.vertex_buffer = Some(vertex_buffer);
	}

	
}

/* Vertex help struct */
#[derive(Copy, Clone)]
pub struct Vertex {
	position: [f32; 2],
	tex_coords: [f32; 2]
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, tex_coords);