pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;

//use crate::window::Window;
use shader::Shader;
use vertex_buffer::VertexBuffer;
use super::window::Window;
use crate::app::glium::{
	self,
	implement_vertex,
	backend::Facade,
	glutin::event_loop::EventLoop,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Struct that handles graphics.
pub struct Graphics {
	pub display:	glium::Display,
	pub imguic:		imgui::Context,
	pub imguiw:		imgui_winit_support::WinitPlatform,
	pub imguir:		imgui_glium_renderer::Renderer,

	pub event_loop:		Option<EventLoop<()>>,
	pub shaders:		Option<Shader>,
	pub vertex_buffer:	Option<glium::VertexBuffer<Vertex>>,
	pub primitive_type:	Option<glium::index::NoIndices>,
}

impl Graphics {
	/// Graphics initialize function. Can be called once.
	/// If you call it again it will panic.
	pub fn initialize() -> Result<Self, &'static str> {
		/* Checks if struct is already initialized */
		#[allow(dead_code)]
		static INITIALIZED: AtomicBool = AtomicBool::new(false);
		if INITIALIZED.load(Ordering::Acquire) {
			return Err("Attempting to initialize graphics twice! Graphics is already initialized!");
		} else {
			INITIALIZED.store(true, Ordering::Release);
		}

		let event_loop = EventLoop::new();
		let window = Window::from(&event_loop, 1024, 768).take_window();

		let mut imgui_context = imgui::Context::create();
		imgui_context.set_ini_filename(None);
		let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
		winit_platform.attach_window(imgui_context.io_mut(), window.window(), imgui_winit_support::HiDpiMode::Rounded);

		imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
		imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
		imgui_context.style_mut().window_rounding = 16.0;

		let settings = std::fs::read_to_string("src/imgui_settings.ini").unwrap();
		imgui_context.load_ini_settings(settings.as_str());

		let display = glium::Display::from_gl_window(window).unwrap();
		let imgui_renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display).unwrap();

		Ok (
			Graphics {
				display: display,
				imguic: imgui_context,
				imguir: imgui_renderer,
				imguiw: winit_platform,
				event_loop: Some(event_loop),
				shaders: None,
				vertex_buffer: None,
				primitive_type: None
			}
		)
	}

	/// Borrow vertex buffer into graphics pipeline.
	pub fn upload_vertex_buffer(&mut self, vertex_buffer: VertexBuffer) {
		self.vertex_buffer = Some(vertex_buffer.vertex_buffer);
		self.primitive_type = Some(vertex_buffer.indices);
	}

	/// Borrow shader into graphics pipeline.
	pub fn upload_shaders(&mut self, shaders: Shader) {
		self.shaders = Some(shaders);
	}

	/// Gives event_loop and removes it from graphics struct
	pub fn take_event_loop(&mut self) -> glium::glutin::event_loop::EventLoop<()> {
		/* Swaps struct variable with returned */
		if let None = self.event_loop {
			panic!("Graphics.event_loop haven't been initialized!")
		} else {
			let mut event_loop = None;
			std::mem::swap(&mut event_loop, &mut self.event_loop);
			event_loop.unwrap()
		}
	}

	/// Gives shaders and removes it from graphics struct
	pub fn take_shaders(&mut self) -> Shader {
		/* Swaps struct variable with returned */
		if let None = self.shaders {
			panic!("Graphics.shaders haven't beed initialized!")
		} else {
			let mut shaders = None;
			std::mem::swap(&mut shaders, &mut self.shaders);
			shaders.unwrap()
		}
	}

	/// Gives vertex_buffer and removes it from graphics struct
	pub fn take_vertex_buffer(&mut self) -> glium::VertexBuffer<Vertex> {
		/* Swaps struct variable with returned */
		if let None = self.vertex_buffer {
			panic!("Graphics.vertex_buffer haven't been initialized!")
		} else {
			let mut vertex_buffer = None;
			std::mem::swap(&mut vertex_buffer, &mut self.vertex_buffer);
			vertex_buffer.unwrap()
		}
	}

	/// Gives primitive_type and removes it from graphics struct
	pub fn take_privitive_type(&mut self) -> glium::index::NoIndices {
		/* Swaps struct variable with returned */
		if let None = self.primitive_type {
			panic!("Graphics.primitive_type haven't been initialized!")
		} else {
			let mut primitive_type = None;
			std::mem::swap(&mut primitive_type, &mut self.primitive_type);
			primitive_type.unwrap()
		}
	}

	#[allow(dead_code)]
	pub fn get_context(&self) -> &std::rc::Rc<glium::backend::Context> {
		self.display.get_context()
	}
}

/// Vertex help struct to OpenGL
#[derive(Copy, Clone)]
pub struct Vertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2]
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, tex_coords);