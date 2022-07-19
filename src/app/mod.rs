pub mod utils;

/* Glium includes */
use glium::{
	glutin::{
		event::{
			Event,
			WindowEvent,
		},
		event_loop::ControlFlow, dpi::PhysicalSize,
	},
	Surface,
	uniform
};

use imgui::InputTextFlags;
/* Other files */
use utils::{
	*,
	user_io::{InputManager, KeyCode},
	graphics::{
		Graphics,
		camera::Camera,
		texture::Texture,
	},
	terrain::chunk::{chunk_array::ChunkArray},
	time::timer::Timer,
	profiler,
};

/// Struct that handles everything.
pub struct App {
	/* Important stuff */
	input_manager: InputManager,
	graphics: Graphics,
	camera: Camera,
	timer: Timer,
	window_size: PhysicalSize<u32>,

	/* Temp voxel */
	chunk_arr: ChunkArray<'static>,

	/* Second layer temporary stuff */
	texture: Texture
}

impl App {
	/// Constructs app struct.
	pub fn new() -> Self {
		/* Graphics initialization */
		let graphics = Graphics::initialize().unwrap();
	
		/* Camera handle */
		let camera = Camera::new().with_position(0.0, 0.0, 2.0);
	
		/* Texture loading */
		let texture = Texture::from("src/image/texture_atlas.png", &graphics.display).unwrap();

		/* Chunk */
		let chunk_arr = ChunkArray::new(&graphics, 4, 1, 4);

		App {
			chunk_arr,
			graphics,
			camera,
			texture,
			timer: Timer::new(),
			input_manager: InputManager::new(),
			window_size: PhysicalSize::new(1024, 768)
		}
	}

	/// Runs app.
	pub fn run(mut self) {
		/* Event/game loop */
		self.graphics.take_event_loop().run(move |event, _, control_flow| {
			self.run_frame_loop(event, control_flow)
		});
	}

	/// Event loop run function
	pub fn run_frame_loop(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
		/* Event handlers */
		self.graphics.imguiw.handle_event(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window(), &event);
		self.input_manager.handle_event(&event, &self.graphics);

		/* This event handler */
		match event {
			/* Window events */
			Event::WindowEvent { event, .. } => match event {
				/* Close event */
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
				},
				WindowEvent::Resized(new_size) => {
					self.camera.aspect_ratio = new_size.height as f32 / new_size.width as f32;
					self.window_size = new_size;
				},
				_ => (),
			},
			Event::MainEventsCleared => {
				self.main_events_cleared(control_flow);
			},
			Event::RedrawRequested(_) => {
				self.redraw_requested();
			},
			Event::NewEvents(_) => {
				self.new_events();			
			},
			_ => ()
		}
	}

	/// Main events cleared.
	fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
		/* Close window is `escape` pressed */
		if self.input_manager.keyboard.just_pressed(KeyCode::Escape) {
			*control_flow = ControlFlow::Exit;
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

		/* Display FPS */
		self.graphics.display.gl_window().window().set_title(format!("Terramine: {0:.0} FPS", self.timer.fps()).as_str());

		/* Update ImGui stuff */
		self.graphics.imguiw
			.prepare_frame(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window())
			.unwrap();

		/* Moves to `RedrawRequested` stage */
		self.graphics.display.gl_window().window().request_redraw();
	}

	/// Prepares the frame.
	fn redraw_requested(&mut self) {
		/* InGui draw data */
		let draw_data = {
			/* Aliasing */
			let camera = &mut self.camera;

			/* Get UI frame renderer */
			let ui = self.graphics.imguic.frame();

			/* Camera control window */
			let mut camera_window = imgui::Window::new("Camera");

			/* Move and resize if pressed I key */
			if !self.input_manager.keyboard.is_pressed(KeyCode::I) {
				camera_window = camera_window
					.resizable(false)
					.movable(false)
					.collapsible(false)
			}

			/* UI building */
			camera_window.build(&ui, || {
				ui.text("Position");
				ui.text(format!("x: {x:.3}, y: {y:.3}, z: {z:.3}", x = camera.get_x(), y = camera.get_y(), z = camera.get_z()));
				ui.text("Rotation");
				ui.text(format!("roll: {roll:.3}, pitch: {pitch:.3}, yaw: {yaw:.3}", roll = camera.roll, pitch = camera.pitch, yaw = camera.yaw));
				ui.separator();
				imgui::Slider::new("Speed", 5.0, 300.0)
					.display_format("%.1f")
					.build(&ui, &mut camera.speed_factor);
				imgui::Slider::new("Speed falloff", 0.0, 1.0)
					.display_format("%.3f")
					.build(&ui, &mut camera.speed_falloff);
				imgui::Slider::new("FOV", 1.0, 180.0)
					.display_format("%.0f")
					.build(&ui, camera.fov.get_degrees_mut());
				camera.fov.update_from_degrees();
			});

			/* Profiler window */
			profiler::update_and_build_window(&ui, &self.timer, &self.input_manager);

			/* Chunk generation window */
			static mut SIZES: [i32; 3] = [7, 1, 7];
			imgui::Window::new("Chunk generator")
				.position_pivot([0.5, 0.5])
				.position([self.window_size.width as f32 * 0.5, self.window_size.height as f32 * 0.5], imgui::Condition::Always)
				.movable(false)
				.size_constraints([150.0, 100.0], [300.0, 200.0])
				.focused(true)
				.save_settings(false)
				.build(&ui, || {
					ui.text("How many?");
					ui.input_int3("Sizes", unsafe { &mut SIZES })
						.auto_select_all(true)
						.enter_returns_true(true)
						.build();
				});

			/* Render UI */
			self.graphics.imguiw.prepare_render(&ui, self.graphics.display.gl_window().window());
			ui.render()
		};

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: self.texture.with_mips(),
			time: self.timer.time(),
			proj: self.camera.get_proj(),
			view: self.camera.get_view()
		};

		/* Actual drawing */
		let mut target = self.graphics.display.draw(); 
		target.clear_all((0.01, 0.01, 0.01, 1.0), 1.0, 0); {
			self.chunk_arr.render(&mut target, &uniforms, &self.camera).unwrap();

			self.graphics.imguir
				.render(&mut target, draw_data)
				.expect("Error rendering imgui");

		} target.finish().unwrap();
	}

	/// Updates things.
	fn new_events(&mut self) {
		/* Update time */
		self.timer.update();
		self.graphics.imguic
			.io_mut()
			.update_delta_time(self.timer.duration());
		
		/* Rotating camera */
		self.camera.update(&mut self.input_manager, self.timer.dt_as_f64());

		/* Input update */
		self.input_manager.update(&self.graphics);		
	}
}
