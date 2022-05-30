mod utils;

/* Glium includes */
use glium::{
	glutin::event::{
		Event,
		WindowEvent,
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

	/* Time stuff */
	let mut time = 0.0;
	let mut last_frame = std::time::Instant::now();
	let mut _dt: f64 = 0.0;

	/* Event/game loop */
	graphics.take_event_loop().run(move |event, _, control_flow| {
		/* Aliasing */
		let window = graphics.display.gl_window();

		/* Event handlers */
		graphics.imguiw.handle_event(graphics.imguic.io_mut(), window.window(), &event);
		input_manager.handle_event(&event);

		/* This event handler */
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
				if input_manager.keyboard.just_pressed(KeyCode::T) {
					if camera.grabbes_cursor {
						camera.grabbes_cursor = false;
						input_manager.mouse.release_cursor(&graphics);
					} else {
						camera.grabbes_cursor = true;
						input_manager.mouse.grab_cursor(&graphics);
					}
				}

	 			/* Update ImGui stuff */
				graphics.imguiw
					.prepare_frame(graphics.imguic.io_mut(), window.window())
					.unwrap();
 
				/* Moves to `RedrawRequested` stage */
				window.window().request_redraw();
	 		},
			glium::glutin::event::Event::RedrawRequested(_) => {
				let draw_data = {
					let ui = graphics.imguic.frame();
					imgui::Window::new("Delta")
						.size([300.0, 150.0], imgui::Condition::FirstUseEver)
						.build(&ui, || {
							ui.text("My delta:");
							ui.text(format!("dx: {0}, dy: {1}", input_manager.mouse.dx, input_manager.mouse.dy));
	
							ui.separator();
	
							ui.text("ImGUI delta");
							ui.text(format!("dx: {0}, dy: {1}", ui.io().mouse_delta[0], ui.io().mouse_delta[1]));
	
							ui.separator();
	
							ui.text("Difference");
							ui.text(format!(
								"ddx: {0}, ddy: {1}",
								ui.io().mouse_delta[0] - input_manager.mouse.dx as f32,
								ui.io().mouse_delta[1] - input_manager.mouse.dy as f32
							));
						});
					imgui::Window::new("Position")
						.size([300.0, 150.0], imgui::Condition::FirstUseEver)
						.build(&ui, || {
							ui.text("My cursor position");
							ui.text(format!("x: {0}, y: {1}", input_manager.mouse.x, input_manager.mouse.y));

							ui.separator();

							ui.text("ImGui cursor position");
							ui.text(format!("x: {0}, y: {1}", ui.io().mouse_pos[0], ui.io().mouse_pos[1]));

							ui.separator();

							ui.text("Difference");
							ui.text(format!(
								"dx: {0}, dy: {1}",
								ui.io().mouse_pos[0] - input_manager.mouse.x as f32,
								ui.io().mouse_pos[1] - input_manager.mouse.y as f32,
							));
						});
	
					graphics.imguiw.prepare_render(&ui, graphics.display.gl_window().window());
					ui.render()
				};

				/* Uniforms set */
				let uniforms = uniform! {
					/* Texture uniform with filtering */
					tex: texture.with_mips(),
					time: time,
					proj: camera.get_proj(),
					view: camera.get_view()
				};

				/* Actual drawing */
				let mut target = graphics.display.draw(); 
				target.clear_color(0.01, 0.01, 0.01, 1.0); {
					target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();

					graphics.imguir
						.render(&mut target, draw_data)
						.expect("error rendering imgui");

				} target.finish().unwrap();
			},
			glium::glutin::event::Event::NewEvents(_) => {
				/* Update time */
				let now = std::time::Instant::now();
				graphics.imguic
					.io_mut()
					.update_delta_time(now.duration_since(last_frame));
				_dt = now.duration_since(last_frame).as_secs_f64();
				last_frame = now;
				time += _dt;
				
				/* Rotating camera */
				camera.update(&mut input_manager, _dt);

				/* Input update */
				input_manager.update(&graphics);				
			},
			_ => ()
		}
	});
}