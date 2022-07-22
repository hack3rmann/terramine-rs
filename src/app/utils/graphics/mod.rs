pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;
pub mod mesh;

//use crate::window::Window;
use shader::Shader;
use vertex_buffer::VertexBuffer;
use super::window::Window;
use crate::app::glium::{
	self,
	implement_vertex,
	backend::Facade,
	glutin::event_loop::EventLoop,
	glutin::event::{
		Event,
		WindowEvent,
	},
	glutin::dpi,
};
use std::{
	sync::atomic::{
		AtomicBool, Ordering
	},
	path::PathBuf,
};

/// Struct that handles graphics.
pub struct Graphics {
	/* Gluim main struct */
	pub display:	glium::Display,

	/* OpenGL pipeline stuff */
	pub event_loop:	Option<EventLoop<()>>,

	/* ImGui stuff */
	pub imguic:		imgui::Context,
	pub imguiw:		imgui_winit_support::WinitPlatform,
	pub imguir:		imgui_glium_renderer::Renderer,
}

impl Graphics {
	/// Graphics initialize function. Can be called once.
	/// If you call it again it will panic.
	pub fn initialize() -> Result<Self, &'static str> {
		/* Validating initialization */
		Self::validate()?;

		/* Glutin event loop */
		let event_loop = EventLoop::new();

		/* Window creation */
		let window = Window::from(&event_loop, 1024, 768).take_window();

		/* Create ImGui context ant set settings file name. */
		let mut imgui_context = imgui::Context::create();
		imgui_context.set_ini_filename(Some(PathBuf::from(r"src/imgui_settings.ini")));

		/* Bound ImGui to winit. */
		let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
		winit_platform.attach_window(imgui_context.io_mut(), window.window(), imgui_winit_support::HiDpiMode::Rounded);

		/* Bad start size fix */
		let dummy_event: Event<()> = Event::WindowEvent {
			window_id: window.window().id(),
			event: WindowEvent::Resized(dpi::PhysicalSize::new(1024, 768))
		};
		winit_platform.handle_event(imgui_context.io_mut(), window.window(), &dummy_event);

		/* Style configuration. */
		imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
		imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
		imgui_context.style_mut().window_rounding = 16.0;

		/* Glium setup. */
		let display = glium::Display::from_gl_window(window).unwrap();

		/* ImGui glium renderer setup. */
		let imgui_renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display).unwrap();

		Ok (
			Graphics {
				display,
				imguic: imgui_context,
				imguir: imgui_renderer,
				imguiw: winit_platform,
				event_loop: Some(event_loop),
			}
		)
	}

	/// Validates initialization.
	fn validate() -> Result<(), &'static str> {
		/* Checks if struct is already initialized */
		#[allow(dead_code)]
		static INITIALIZED: AtomicBool = AtomicBool::new(false);
		if INITIALIZED.load(Ordering::Acquire) {
			return Err("Attempting to initialize graphics twice! Graphics is already initialized!");
		} else {
			Ok(INITIALIZED.store(true, Ordering::Release))
		}
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

	#[allow(dead_code)]
	pub fn get_context(&self) -> &std::rc::Rc<glium::backend::Context> {
		self.display.get_context()
	}
}

/// Vertex help struct to OpenGL
#[derive(Copy, Clone)]
pub struct Vertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
	pub light: f32
}

/* Implement Vertex struct as main vertex struct in glium */
implement_vertex!(Vertex, position, tex_coords, light);