pub mod utils;

/* Glium includes */
use glium::{
	glutin::event::{
		Event,
		WindowEvent,
	},
	Surface,
	uniform
};

/* File stream use */
use std::io::prelude::*;

/* Other files */
use utils::{
	*,
	user_io::{InputManager, KeyCode},
	graphics::{
		Graphics,
		camera::Camera,
		shader::Shader,
		texture::Texture,
		vertex_buffer::VertexBuffer,
	},
};

/// Struct that handles everything.
pub struct App {
	/* Important stuff */
	input_manager: InputManager,
	graphics: Graphics,
	camera: Camera,

	/* Important but TODO stuff */
	time: f64,
	last_frame: std::time::Instant,
	dt: f64,

	/* First layer temporary stuff */
	vertex_buffer: glium::VertexBuffer<graphics::Vertex>,
	indices: glium::index::NoIndices,
	shaders: Shader,

	/* Second layer temporary stuff */
	texture: Texture
}

impl App {
	/// Constructs app struct.
	pub fn new() -> Self {
		/* Graphics initialization */
		let mut graphics = Graphics::initialize().unwrap();
	
		/* Camera handle */
		let mut camera = Camera::new();
	
		/* Texture loading */
		let texture = Texture::from("src/image/grass_top_separ.png", &graphics.display).unwrap();
	
		/* Vertex buffer loading */
		let vertex_buffer = VertexBuffer::default(&graphics);
		vertex_buffer.bind(&mut graphics);
	
		/* Shader program */
		let shaders = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
		graphics.upload_shaders(shaders);
	
		/* Camera preposition */
		camera.set_position(0.0, 0.0, 2.0);

		/* Destruct: */
		let vertex_buffer = graphics.take_vertex_buffer();
		let indices = graphics.take_privitive_type();
		let shaders = graphics.take_shaders();

		App {
			input_manager: InputManager::new(),
			graphics: graphics,
			camera: camera,
			time: 0.0,
			last_frame: std::time::Instant::now(),
			dt: 0.0,
			vertex_buffer: vertex_buffer,
			indices: indices,
			shaders: shaders,
			texture: texture
		}
	}

	/// Runs app.
	pub fn run(mut self) {
		/* Event/game loop */
		self.graphics.take_event_loop().run(move |event, _, control_flow| {
			/* Event handlers */
			self.graphics.imguiw.handle_event(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window(), &event);
			self.input_manager.handle_event(&event, &self.graphics);

			/* This event handler */
			match event {
				/* Window events */
				Event::WindowEvent { event, .. } => match event {
					/* Close event */
					WindowEvent::CloseRequested => {
						*control_flow = glium::glutin::event_loop::ControlFlow::Exit;
						let mut buf: String = Default::default();
						self.graphics.imguic.save_ini_settings(&mut buf);

						let mut file = std::fs::File::create("src/imgui_settings.ini").unwrap();
						file.write_all(buf.as_bytes()).unwrap();
					},
					WindowEvent::Resized(new_size) => {
						self.camera.aspect_ratio = new_size.height as f32 / new_size.width as f32;
					},
					_ => (),
				},
				Event::MainEventsCleared => {
					self.main_events_cleared(control_flow);
				},
				glium::glutin::event::Event::RedrawRequested(_) => {
					self.redraw_requested();
				},
				glium::glutin::event::Event::NewEvents(_) => {
					self.new_events();			
				},
				_ => ()
			}
		});
	}

	/// Main events cleared.
	fn main_events_cleared(&mut self, control_flow: &mut glium::glutin::event_loop::ControlFlow) {
		/* Close window is `escape` pressed */
		if self.input_manager.keyboard.just_pressed(KeyCode::Escape) {
			*control_flow = glium::glutin::event_loop::ControlFlow::Exit;
			let mut buf: String = Default::default();
			self.graphics.imguic.save_ini_settings(&mut buf);

			let mut file = std::fs::File::create("src/imgui_settings.ini").unwrap();
			file.write_all(buf.as_bytes()).unwrap();
		}

		/* Control camera by user input */
		if self.input_manager.keyboard.just_pressed(KeyCode::T) {
			if self.camera.grabbes_cursor {
				self.camera.grabbes_cursor = false;
				self.input_manager.mouse.release_cursor(&self.graphics);
			} else {
				self.camera.grabbes_cursor = true;
				self.input_manager.mouse.grab_cursor(&self.graphics);
			}
		}

		/* Update ImGui stuff */
		self.graphics.imguiw
			.prepare_frame(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window())
			.unwrap();

		/* Moves to `RedrawRequested` stage */
		self.graphics.display.gl_window().window().request_redraw();
	}

	/// Prepares the frame.
	fn redraw_requested(&mut self) {
		let draw_data = {
			let camera = &mut self.camera;

			let ui = self.graphics.imguic.frame();
			imgui::Window::new("Camera")
				.resizable(false)
				.movable(false)
				.collapsible(false)
				.build(&ui, || {
					ui.text("Position");
					ui.text(format!("x: {0:.3}, y: {1:.3}, z: {2:.3}", camera.get_x(), camera.get_y(), camera.get_z()));
					ui.separator();
					ui.text("Speed factor");
					imgui::Slider::new("Camera speed", 5.0, 50.0)
						.display_format("%.1f")
						.build(&ui, &mut camera.speed);
					imgui::Slider::new("Camera fov", 0.0, std::f32::consts::PI * 4.0)
						.display_format("%.2f")
						.build(&ui, &mut camera.fov);
				});

			self.graphics.imguiw.prepare_render(&ui, self.graphics.display.gl_window().window());
			ui.render()
		};

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: self.texture.with_mips(),
			time: self.time,
			proj: self.camera.get_proj(),
			view: self.camera.get_view()
		};

		let params = glium::DrawParameters {
			depth: glium::Depth {
				test: glium::DepthTest::Overwrite,
				write: true,
				.. Default::default()
			},
			backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
			.. Default::default()
		};

		/* Actual drawing */
		let mut target = self.graphics.display.draw(); 
		target.clear_color(0.01, 0.01, 0.01, 1.0);
		target.clear_depth(0.0); {
			target.draw(&self.vertex_buffer, &self.indices, &self.shaders.program, &uniforms, &params).unwrap();

			self.graphics.imguir
				.render(&mut target, draw_data)
				.expect("error rendering imgui");

		} target.finish().unwrap();
	}

	/// Updates things.
	fn new_events(&mut self) {
		/* Update time */
		let now = std::time::Instant::now();
		self.graphics.imguic
			.io_mut()
			.update_delta_time(now.duration_since(self.last_frame));
		self.dt = now.duration_since(self.last_frame).as_secs_f64();
		self.last_frame = now;
		self.time += self.dt;
		
		/* Rotating camera */
		self.camera.update(&mut self.input_manager, self.dt);

		/* Input update */
		self.input_manager.update(&self.graphics);		
	}
}