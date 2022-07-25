pub mod utils;

use {
	/* Glium includes */
	glium::{
		glutin::{
			event::{
				Event,
				WindowEvent,
			},
			event_loop::ControlFlow, dpi::PhysicalSize,
		},
		Surface,
		uniform
	},

	/* Other files */
	utils::{
		*,
		user_io::{InputManager, KeyCode},
		graphics::{
			Graphics,
			camera::Camera,
			texture::Texture,
			Vertex,
		},
		terrain::chunk::chunk_array::{
			MeshlessChunkArray,
			MeshedChunkArray,
		},
		time::timer::Timer,
		profiler,
		concurrency::promise::Promise,
		runtime::prelude::*,
	},

	crate::app::utils::concurrency::loading::Loading,
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
	chunk_arr: Option<MeshedChunkArray<'static>>,

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

		App {
			chunk_arr: None,
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
			runtime().block_on(async {
				self.run_frame_loop(event, control_flow).await
			})
		});
	}

	/// Event loop run function
	pub async fn run_frame_loop<'e>(&mut self, event: Event<'e, ()>, control_flow: &mut ControlFlow) {
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
		/* Chunk generation flag */
		let mut generate_chunks = false;
		static mut SIZES: [i32; 3] = [7, 1, 7];
		static mut GENERATION_PERCENTAGE: Loading = Loading::none();

		/* InGui draw data */
		let draw_data = {
			/* Get UI frame renderer */
			let ui = self.graphics.imguic.frame();

			/* Camera window */
			self.camera.spawn_control_window(&ui, &mut self.input_manager);

			/* Profiler window */
			profiler::update_and_build_window(&ui, &self.timer, &self.input_manager);

			/* Chunk generation window */
			Self::spawn_chunk_generation_window(
				&ui, self.chunk_arr.is_some(), self.window_size.width as f32, self.window_size.height as f32,
				&mut generate_chunks, unsafe { &mut SIZES }, unsafe { GENERATION_PERCENTAGE }
			);

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
			if let Some(chunk_arr) = self.chunk_arr.as_mut() {
				chunk_arr.render(&mut target, &uniforms, &self.camera).unwrap();
			}

			self.graphics.imguir
				.render(&mut target, draw_data)
				.expect("Error rendering imgui");

		} target.finish().unwrap();

		/* Chunk reciever */
		static mut CHUNKS_PROMISE: Option<Promise<(MeshlessChunkArray, Vec<Vec<Vertex>>)>> = None;
		static mut PERCENTAGE_PROMISE: Option<Promise<Loading>> = None;
		if generate_chunks {
			/* Dimensions shortcut */
			let (width, height, depth) = unsafe {
				(SIZES[0] as usize, SIZES[1] as usize, SIZES[2] as usize)
			};

			/* Get receivers */
			let (array, percentage) = MeshlessChunkArray::generate(width, height, depth);

			/* Write to statics */
			unsafe {
				(CHUNKS_PROMISE, PERCENTAGE_PROMISE) = (Some(array), Some(percentage))
			};
		}

		/* If array recieved then store it in self */
		if let Some(promise) = unsafe { CHUNKS_PROMISE.as_ref() } {
			promise.poll_do_cleanup(|array| {
				/* Apply meshes to chunks */
				let array = {
					let (array, meshes) = array;
					array.upgrade(&self.graphics, meshes)
				};

				/* Move result */
				self.chunk_arr = Some(array);
			}, || unsafe { CHUNKS_PROMISE = None });
		}

		/* Receive percentage */
		if let Some(promise) = unsafe { PERCENTAGE_PROMISE.as_ref() } {
			if let Some(percent) = promise.iter().last() {
				unsafe { GENERATION_PERCENTAGE = percent } 
			}
		}
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

	/// Spawns chunk generation window.
	pub fn spawn_chunk_generation_window(ui: &imgui::Ui, inited: bool, width: f32, height: f32, generate_chunks: &mut bool, sizes: &mut [i32; 3], gen_percent: Loading) {
		if !inited {
			imgui::Window::new("Chunk generator")
				.position_pivot([0.5, 0.5])
				.position([width * 0.5, height as f32 * 0.5], imgui::Condition::Always)
				.movable(false)
				.size_constraints([150.0, 100.0], [300.0, 200.0])
				.always_auto_resize(true)
				.focused(true)
				.save_settings(false)
				.build(&ui, || {
					ui.text("How many?");
					ui.input_int3("Sizes", sizes)
						.auto_select_all(true)
						.enter_returns_true(true)
						.build();
					*generate_chunks = ui.button("Generate");

					if gen_percent != Loading::none() {
						imgui::ProgressBar::new(gen_percent.percent as f32)
							.overlay_text(format!(
								"{state} ({percent:.1}%)",
								percent = gen_percent.percent * 100.0,
								state = gen_percent.state
							))
							.build(&ui);
					}
				});
		}
	}
}
