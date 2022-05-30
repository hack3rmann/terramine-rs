mod utils;

/* Glium includes */
use glium::{
	glutin::event::{
		Event,
		WindowEvent,
		ElementState,
		DeviceEvent
	},
	Surface,
	uniform
};

/* Other files */
use utils::{
	*,
	user_io::{InputManager, KeyCode},
	window::Window,
	graphics::{
		Graphics,
		camera::Camera,
		shader::Shader,
		texture::Texture,
		vertex_buffer::VertexBuffer,
	},
};

fn main() {
	/* Keyboard init */
	let mut input_manager = InputManager::new();

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

	/* Camera preposition */
	camera.set_position(0.0, 0.0, 2.0);

	let mut is_cursor_grabbed = false;

	let mut time = 0.0;
	let mut last_frame = std::time::Instant::now();
	let mut dt: f64 = 0.0;
	graphics.take_event_loop().run(move |event, _, control_flow| {
		let window = graphics.display.gl_window();
		graphics.imguiw.handle_event(graphics.imguic.io_mut(), window.window(), &event);
		input_manager.handle_event(&event, &graphics);

		match event {
			/* Window events */
	        Event::WindowEvent { event, .. } => match event {
	 			/* Close event */
	            WindowEvent::CloseRequested => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
	             _ => (),
	        },
	 		Event::MainEventsCleared => {
	 			/* Close window is `escape` pressed */
	 			if input_manager.keyboard.just_pressed(KeyCode::Escape) {
	 				Window::exit(control_flow);
	 			}

	 			/* Control camera by user input */
				if input_manager.keyboard.just_pressed(KeyCode::Q) { is_cursor_grabbed = !is_cursor_grabbed; }
	 			if input_manager.keyboard.is_pressed(KeyCode::W)		{ camera.move_pos( dt,  0.0,    0.0); }
	 			if input_manager.keyboard.is_pressed(KeyCode::S)		{ camera.move_pos(-dt,  0.0,    0.0); }
	 			if input_manager.keyboard.is_pressed(KeyCode::D)		{ camera.move_pos( 0.0,    0.0,   -dt); }
	 			if input_manager.keyboard.is_pressed(KeyCode::A)		{ camera.move_pos( 0.0,    0.0,    dt); }
	 			if input_manager.keyboard.is_pressed(KeyCode::LShift)	{ camera.move_pos( 0.0,   -dt,  0.0); }
	 			if input_manager.keyboard.is_pressed(KeyCode::Space)	{ camera.move_pos( 0.0,    dt,  0.0); }
	 			if input_manager.mouse.just_left_pressed() {
	 				camera.set_position(0.0, 0.0, 2.0);
	 				camera.reset_rotation();
	 			}

	 			/* Update ImGui stuff */
				graphics.imguiw
					.prepare_frame(graphics.imguic.io_mut(), window.window())
					.unwrap();
 
				window.window().request_redraw();
	 		},
			glium::glutin::event::Event::RedrawRequested(_) => {
				let ui = graphics.imguic.frame();
				imgui::Window::new("Delta")
					.size([300.0, 100.0], imgui::Condition::FirstUseEver)
					.build(&ui, || {
						ui.text("My delta:");
						ui.text(format!("dx: {0:.5}, dy: {1:.5}", input_manager.mouse.dx, input_manager.mouse.dy));

						ui.separator();

						ui.text("ImGUI delta");
						ui.text(format!("dx: {0:.5}, dy: {1:.5}", ui.io().mouse_delta[0], ui.io().mouse_delta[1]))
					});

				graphics.imguiw.prepare_render(&ui, window.window());
				let draw_data = ui.render();

				/* Uniforms set */
				let uniforms = uniform! {
					/* Texture uniform with filtering */
					tex: texture.with_mips(),
					time: time,
					proj: camera.get_proj(),
					view: camera.get_view()
				};

				let mut target = graphics.display.draw(); 
				target.clear_color(0.01, 0.01, 0.01, 1.0); {
					target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();

					graphics.imguir
						.render(&mut target, draw_data)
						.expect("error rendering imgui");

				} target.finish().unwrap();
			},
			glium::glutin::event::Event::NewEvents(_) => {
				let now = std::time::Instant::now();
				graphics.imguic
					.io_mut()
					.update_delta_time(now.duration_since(last_frame));
				dt = now.duration_since(last_frame).as_secs_f64();
				last_frame = now;
				time += dt;
				
				/* Rotating camera */
				camera.rotate(
					-input_manager.mouse.dy * dt * 0.2,
					 input_manager.mouse.dx * dt * 0.2,
					 0.0
				);

				input_manager.update();

				// camera.rotate(
				// 	-graphics.imguic.io().mouse_delta[1] as f64 * dt * 0.2,
				// 	 graphics.imguic.io().mouse_delta[0] as f64 * dt * 0.2,
				// 	 0.0
				// );

				if is_cursor_grabbed {
					graphics.display.gl_window().window().set_cursor_position(
						glium::glutin::dpi::LogicalPosition::new(1024 / 2, 768 / 2)
					).unwrap();
				}
			},
			_ => ()
		}
	});
}